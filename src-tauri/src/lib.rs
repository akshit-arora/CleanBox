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

#[derive(Serialize, Deserialize, Default)]
pub struct GeneralSettings {
    pub currency_symbol: String,
    pub number_locale: String, // "en-US", "en-IN"
    pub decimals: bool,
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
async fn get_recent_emails(
    pool: State<'_, Pool<Sqlite>>,
    limit: i64,
) -> Result<Vec<db::Email>, String> {
    let emails =
        sqlx::query_as::<_, db::Email>("SELECT * FROM emails ORDER BY received_at DESC LIMIT ?")
            .bind(limit)
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

// use db::KanbanStage;
// use std::str::FromStr; // Unused now

// Logic extracted for testing
pub async fn move_email_db(pool: &Pool<Sqlite>, id: String, stage: String) -> Result<(), String> {
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
    sqlx::query("UPDATE emails SET kanban_status = ? WHERE id = ?")
        .bind(&stage)
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Log
    println!("Moved email {} to {}", id, stage);

    Ok(())
}

#[tauri::command]
async fn move_email(
    pool: State<'_, Pool<Sqlite>>,
    id: String,
    stage: String,
) -> Result<(), String> {
    move_email_db(&*pool, id, stage).await
}

#[tauri::command]
async fn update_email_status(
    pool: State<'_, Pool<Sqlite>>,
    id: String,
    status: String,
) -> Result<(), String> {
    // Determine the status string to save
    // If it maps to a known stage, cool, but we now support arbitrary IDs.
    // However, for compatibility with old move_email_db which takes KanbanStage, we need to be careful.
    // Wait, move_email_db takes KanbanStage. I should refactor move_email_db to take string, or create a new one.

    // Let's refactor move_email_db to take &str.
    // But first, let's just do a direct query here to avoid breaking too much at once,
    // OR create a helper `move_email_any_status`.

    // 1. Validation: Ensure ID exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM emails WHERE id = ?)")
        .bind(&id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Email with ID {} not found.", id));
    }

    // 2. Update Status
    sqlx::query("UPDATE emails SET kanban_status = ? WHERE id = ?")
        .bind(&status)
        .bind(&id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Logic extracted for testing
pub async fn get_kanban_config_db(pool: &Pool<Sqlite>) -> Result<Vec<db::KanbanColumn>, String> {
    // 1. Try to fetch from settings
    let row: Option<String> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = 'kanban_config'")
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    if let Some(json_str) = row {
        let columns: Vec<db::KanbanColumn> = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse kanban config: {}", e))?;
        return Ok(columns);
    }

    // 2. Return Default if not found
    Ok(vec![
        db::KanbanColumn {
            id: "inbox".to_string(),
            title: "Inbox".to_string(),
            color: "bg-blue-500/10 text-blue-500".to_string(),
        },
        db::KanbanColumn {
            id: "action".to_string(),
            title: "Action".to_string(),
            color: "bg-orange-500/10 text-orange-500".to_string(),
        },
        db::KanbanColumn {
            id: "waiting".to_string(),
            title: "Waiting".to_string(),
            color: "bg-yellow-500/10 text-yellow-500".to_string(),
        },
        db::KanbanColumn {
            id: "done".to_string(),
            title: "Done".to_string(),
            color: "bg-green-500/10 text-green-500".to_string(),
        },
    ])
}

#[tauri::command]
async fn get_kanban_config(pool: State<'_, Pool<Sqlite>>) -> Result<Vec<db::KanbanColumn>, String> {
    get_kanban_config_db(&*pool).await
}

#[tauri::command]
async fn save_kanban_config(
    pool: State<'_, Pool<Sqlite>>,
    columns: Vec<db::KanbanColumn>,
) -> Result<(), String> {
    let json_str = serde_json::to_string(&columns).map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO settings (key, value) VALUES ('kanban_config', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind(json_str)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Logic extracted for testing
pub async fn get_kanban_board_db(pool: &Pool<Sqlite>) -> Result<db::KanbanBoard, String> {
    // 1. Fetch Config
    let columns_config = get_kanban_config_db(pool).await?;

    // 2. Fetch all 'WORKFLOW' emails
    let emails = sqlx::query_as::<_, db::Email>(
        "SELECT * FROM emails WHERE view_mode = 'WORKFLOW' ORDER BY received_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 3. Initialize Board with empty buckets
    let mut column_data: Vec<db::KanbanColumnData> = columns_config
        .iter()
        .map(|c| db::KanbanColumnData {
            id: c.id.clone(),
            title: c.title.clone(),
            color: c.color.clone(),
            emails: Vec::new(),
        })
        .collect();

    // 4. Distribute emails
    for email in emails {
        // Find matching column, default to first column (usually Inbox) if not found
        // The comparison should be case-insensitive to be safe?
        // Or strictly matching ID. Let's assume ID match.
        // We normalize to lowercase for ID matching just in case, but IDs should be stable.

        let target_id = email.kanban_status.to_lowercase();

        if let Some(col) = column_data
            .iter_mut()
            .find(|c| c.id.to_lowercase() == target_id)
        {
            col.emails.push(email);
        } else if let Some(first) = column_data.first_mut() {
            // Fallback to first column
            first.emails.push(email);
        }
    }

    Ok(db::KanbanBoard {
        columns: column_data,
    })
}

#[tauri::command]
async fn get_kanban_board(pool: State<'_, Pool<Sqlite>>) -> Result<db::KanbanBoard, String> {
    get_kanban_board_db(&*pool).await
}

#[tauri::command]
async fn get_chat_threads(pool: State<'_, Pool<Sqlite>>) -> Result<Vec<db::ChatThread>, String> {
    // We use a subquery to ensure we get the subject from the row with the MAX(received_at)
    let threads = sqlx::query_as::<_, db::ChatThread>(
        "SELECT 
            sender as sender_name, 
            sender as sender_email, 
            subject as latest_subject, 
            received_at as last_message_time
         FROM emails e1
         WHERE view_mode = 'CHAT' 
           AND received_at = (
               SELECT MAX(received_at) 
               FROM emails e2 
               WHERE e2.sender = e1.sender AND e2.view_mode = 'CHAT'
           )
         GROUP BY sender
         ORDER BY last_message_time DESC",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(threads)
}

#[tauri::command]
async fn get_thread_messages(
    pool: State<'_, Pool<Sqlite>>,
    email: String,
) -> Result<Vec<db::Email>, String> {
    let emails = sqlx::query_as::<_, db::Email>(
        "SELECT * FROM emails WHERE sender = ? AND view_mode = 'CHAT' ORDER BY received_at ASC",
    )
    .bind(email)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(emails)
}

#[tauri::command]
async fn get_ledger_stats(_pool: State<'_, Pool<Sqlite>>) -> Result<db::LedgerStats, String> {
    // Return dummy data for now as requested
    Ok(db::LedgerStats {
        total_income: 15000.0,
        total_expense: 4300.0,
        balance: 10700.0,
    })
}

#[tauri::command]
async fn get_transactions(pool: State<'_, Pool<Sqlite>>) -> Result<Vec<db::Email>, String> {
    let emails = sqlx::query_as::<_, db::Email>(
        "SELECT * FROM emails WHERE view_mode = 'LEDGER' ORDER BY received_at DESC",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(emails)
}

#[tauri::command]
async fn save_general_settings(
    pool: State<'_, Pool<Sqlite>>,
    settings: GeneralSettings,
) -> Result<(), String> {
    let queries = vec![
        ("currency_symbol", settings.currency_symbol),
        ("number_locale", settings.number_locale),
        ("show_decimals", settings.decimals.to_string()),
    ];

    for (key, value) in queries {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(key)
            .bind(value)
            .execute(&*pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn get_general_settings(pool: State<'_, Pool<Sqlite>>) -> Result<GeneralSettings, String> {
    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut settings = GeneralSettings {
        currency_symbol: "$".to_string(),
        number_locale: "en-US".to_string(),
        decimals: true,
    };

    for row in rows {
        let key: String = row.try_get("key").unwrap_or_default();
        let value: String = row.try_get("value").unwrap_or_default();

        match key.as_str() {
            "currency_symbol" => settings.currency_symbol = value,
            "number_locale" => settings.number_locale = value,
            "show_decimals" => settings.decimals = value.parse().unwrap_or(true),
            _ => {}
        }
    }

    Ok(settings)
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
            get_recent_emails,
            analyze_and_create_rule,
            move_email,
            get_kanban_board,
            update_email_status,
            get_kanban_config,
            save_kanban_config,
            get_chat_threads,
            get_thread_messages,
            get_ledger_stats,
            get_transactions,
            save_general_settings,
            get_general_settings,
            imap_sync::fetch_inbox_top
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
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

        sqlx::query("CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);")
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
        // logic moved to check against dynamic but underlying db is still string.
        // update_email_status logic actually uses KanbanStage::from_str internally still?
        // Ah, I need to check remove strict KanbanStage check in update_email_status in lib.rs if I want fully dynamic.
        // But for this test, let's just use string update directly or helper.

        move_email_db(&pool, "123".to_string(), "ACTION".to_string())
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
        sqlx::query("INSERT INTO emails (id, sender, subject, view_mode, kanban_status, received_at) VALUES ('1', 's', 's', 'WORKFLOW', 'inbox', '2023')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO emails (id, sender, subject, view_mode, kanban_status, received_at) VALUES ('2', 's', 's', 'WORKFLOW', 'action', '2023')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO emails (id, sender, subject, view_mode, kanban_status, received_at) VALUES ('3', 's', 's', 'WORKFLOW', 'done', '2023')").execute(&pool).await.unwrap();

        let board = get_kanban_board_db(&pool).await.unwrap();

        // Defaults: Inbox, Action, Waiting, Done
        assert_eq!(board.columns.len(), 4);

        // Inbox
        assert_eq!(board.columns[0].id, "inbox");
        assert_eq!(board.columns[0].emails.len(), 1);

        // Action
        assert_eq!(board.columns[1].id, "action");
        assert_eq!(board.columns[1].emails.len(), 1);

        // Done
        assert_eq!(board.columns[3].id, "done");
        assert_eq!(board.columns[3].emails.len(), 1);
    }
}
