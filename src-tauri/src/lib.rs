use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use tauri::{Manager, State};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Deserialize, Default)]
pub struct ImapConfig {
    pub imap_host: String,
    pub imap_port: String,
    pub imap_user: String,
    pub imap_password: String,
    pub smtp_host: String,
    pub smtp_port: String,
    pub smtp_user: String,
    pub smtp_password: String,
}

#[tauri::command]
async fn save_imap_config(
    pool: State<'_, Pool<Sqlite>>,
    config: ImapConfig,
) -> Result<String, String> {
    // 1. Save Secrets to OS Keychain
    secure_store::save_secret("imap_password", &config.imap_password)?;
    secure_store::save_secret("smtp_password", &config.smtp_password)?;

    // 2. Save Non-Sensitive Settings to SQLite
    let queries = vec![
        ("imap_host", config.imap_host),
        ("imap_port", config.imap_port),
        ("imap_user", config.imap_user),
        ("smtp_host", config.smtp_host),
        ("smtp_port", config.smtp_port),
        ("smtp_user", config.smtp_user),
    ];

    for (key, value) in queries {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(key)
            .bind(value)
            .execute(&*pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok("Configuration saved successfully!".to_string())
}

#[tauri::command]
async fn get_imap_config(pool: State<'_, Pool<Sqlite>>) -> Result<ImapConfig, String> {
    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut config = ImapConfig::default();

    // 1. Fetch from DB
    for row in rows {
        let key: String = row.try_get("key").unwrap_or_default();
        let value: String = row.try_get("value").unwrap_or_default();

        match key.as_str() {
            "imap_host" => config.imap_host = value,
            "imap_port" => config.imap_port = value,
            "imap_user" => config.imap_user = value,
            "smtp_host" => config.smtp_host = value,
            "smtp_port" => config.smtp_port = value,
            "smtp_user" => config.smtp_user = value,
            _ => {}
        }
    }

    // 2. Fetch from Secure Store
    config.imap_password = secure_store::get_secret("imap_password").unwrap_or_default();
    config.smtp_password = secure_store::get_secret("smtp_password").unwrap_or_default();

    Ok(config)
}

pub mod db;
pub mod engine;
pub mod imap_sync;
pub mod secure_store;

#[tauri::command]
async fn get_emails(
    pool: State<'_, Pool<Sqlite>>,
    view_mode: String,
) -> Result<Vec<db::Email>, String> {
    let emails = sqlx::query_as::<_, db::Email>(
        "SELECT * FROM emails WHERE view_mode = ? ORDER BY received_at DESC",
    )
    .bind(view_mode)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(emails)
}

#[tauri::command]
async fn analyze_and_create_rule(
    pool: State<'_, Pool<Sqlite>>,
    email_id: String,
    target_view_mode: String,
) -> Result<String, String> {
    // 1. Fetch Target Email
    let target_email = sqlx::query_as::<_, db::Email>("SELECT * FROM emails WHERE id = ?")
        .bind(&email_id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Email not found")?;

    // 2. Fetch Related Emails (Same Sender)
    let related_emails = sqlx::query_as::<_, db::Email>("SELECT * FROM emails WHERE sender = ?")
        .bind(&target_email.sender)
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Generate Rule
    let rule = engine::generate_smart_rule(&target_email, &related_emails, target_view_mode);

    // 4. Insert Rule
    sqlx::query("INSERT INTO custom_rules (name, sender_filter, subject_filter, body_filter, target_view_mode, priority) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&rule.name)
        .bind(&rule.sender_filter)
        .bind(&rule.subject_filter)
        .bind(&rule.body_filter)
        .bind(&rule.target_view_mode)
        .bind(rule.priority)
        .execute(&*pool)
        .await
        .map_err(|e| format!("Failed to insert rule: {}", e))?;

    // 5. Re-classify related emails
    let all_rules = sqlx::query_as::<_, engine::CustomRule>(
        "SELECT * FROM custom_rules ORDER BY priority DESC",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut updated_count = 0;
    for email in related_emails {
        // Note: Using body_preview as we might not have full body in DB.
        // Only fetch_inbox_top has full body temporarily.
        // Ideally we should store full body if we want robust re-classification on body.
        let (new_mode, _) = engine::classify_email(
            &email.subject,
            &email.sender,
            &email.body_preview,
            &all_rules,
        );

        if new_mode.as_str() != email.view_mode {
            sqlx::query("UPDATE emails SET view_mode = ? WHERE id = ?")
                .bind(new_mode.as_str())
                .bind(&email.id)
                .execute(&*pool)
                .await
                .map_err(|e| e.to_string())?;
            updated_count += 1;
        }
    }

    Ok(format!(
        "Rule '{}' created. Re-classified {} emails.",
        rule.name, updated_count
    ))
}

use db::KanbanStage;
use std::str::FromStr;

// Logic extracted for testing
pub async fn move_email_db(
    pool: &Pool<Sqlite>,
    id: String,
    stage: KanbanStage,
) -> Result<(), String> {
    // 1. Validation: Ensure ID exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM emails WHERE id = ?)")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Email with ID {} not found.", id));
    }

    // 2. Update Status
    let stage_str = stage.to_string();
    sqlx::query("UPDATE emails SET kanban_status = ? WHERE id = ?")
        .bind(&stage_str)
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Log
    println!("Moved email {} to {}", id, stage_str);

    Ok(())
}

#[tauri::command]
async fn move_email(
    pool: State<'_, Pool<Sqlite>>,
    id: String,
    stage: KanbanStage,
) -> Result<(), String> {
    move_email_db(&*pool, id, stage).await
}

// Logic extracted for testing
pub async fn get_kanban_board_db(pool: &Pool<Sqlite>) -> Result<db::KanbanBoardData, String> {
    // 1. Fetch all 'WORKFLOW' emails
    let emails = sqlx::query_as::<_, db::Email>(
        "SELECT * FROM emails WHERE view_mode = 'WORKFLOW' ORDER BY received_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 2. Sort into buckets in Rust
    let mut board = db::KanbanBoardData {
        inbox: Vec::new(),
        action: Vec::new(),
        waiting: Vec::new(),
        done: Vec::new(),
    };

    for email in emails {
        // Parse the status string back to Enum to decide where it goes
        // We use FromStr which defaults to Inbox on failure
        let stage = KanbanStage::from_str(&email.kanban_status).unwrap_or(KanbanStage::Inbox);

        match stage {
            KanbanStage::Inbox => board.inbox.push(email),
            KanbanStage::Action => board.action.push(email),
            KanbanStage::Waiting => board.waiting.push(email),
            KanbanStage::Done => board.done.push(email),
        }
    }

    Ok(board)
}

#[tauri::command]
async fn get_kanban_board(pool: State<'_, Pool<Sqlite>>) -> Result<db::KanbanBoardData, String> {
    get_kanban_board_db(&*pool).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async {
                let pool = db::init_db().await;
                app.manage(pool);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            save_imap_config,
            get_imap_config,
            get_emails,
            analyze_and_create_rule,
            move_email,
            get_kanban_board,
            imap_sync::fetch_inbox_top
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::KanbanStage;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE emails (
                id TEXT PRIMARY KEY,
                sender TEXT NOT NULL,
                subject TEXT NOT NULL,
                body_preview TEXT,
                view_mode TEXT NOT NULL,
                kanban_status TEXT NOT NULL,
                amount REAL,
                merchant TEXT,
                received_at TEXT NOT NULL
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_move_email() {
        let pool = setup_test_db().await;
        // Insert dummy email
        sqlx::query("INSERT INTO emails (id, sender, subject, view_mode, kanban_status, received_at) VALUES ('123', 'me', 'test', 'WORKFLOW', 'INBOX', '2023-01-01')")
            .execute(&pool)
            .await
            .unwrap();

        // Move to Action
        move_email_db(&pool, "123".to_string(), KanbanStage::Action)
            .await
            .unwrap();

        // Verify
        let status: String =
            sqlx::query_scalar("SELECT kanban_status FROM emails WHERE id = '123'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(status, "ACTION");
    }

    #[tokio::test]
    async fn test_get_kanban_board() {
        let pool = setup_test_db().await;

        // Insert mixed emails
        sqlx::query("INSERT INTO emails (id, sender, subject, view_mode, kanban_status, received_at) VALUES ('1', 's', 's', 'WORKFLOW', 'INBOX', '2023')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO emails (id, sender, subject, view_mode, kanban_status, received_at) VALUES ('2', 's', 's', 'WORKFLOW', 'ACTION', '2023')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO emails (id, sender, subject, view_mode, kanban_status, received_at) VALUES ('3', 's', 's', 'WORKFLOW', 'DONE', '2023')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO emails (id, sender, subject, view_mode, kanban_status, received_at) VALUES ('4', 's', 's', 'FEED', 'INBOX', '2023')").execute(&pool).await.unwrap(); // Should be ignored

        let board = get_kanban_board_db(&pool).await.unwrap();

        assert_eq!(board.inbox.len(), 1);
        assert_eq!(board.action.len(), 1);
        assert_eq!(board.done.len(), 1);
        assert_eq!(board.waiting.len(), 0);

        assert_eq!(board.inbox[0].id, "1");
        assert_eq!(board.action[0].id, "2");
    }
}
