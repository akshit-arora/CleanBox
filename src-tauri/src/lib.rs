use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use tauri::{Manager, State};

#[derive(Serialize, Deserialize)]
pub struct ImapConfigStruct {
    pub host: String,
    pub port: u16,
    pub email: String,
    pub password: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn save_imap_config(
    pool: State<'_, Pool<Sqlite>>,
    host: String,
    port: u16,
    email: String,
    password: String,
) -> Result<String, String> {
    let queries = vec![
        ("imap_host", host),
        ("imap_port", port.to_string()),
        ("imap_user", email),
        ("imap_password", password),
    ];

    for (key, value) in queries {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2")
            .bind(key)
            .bind(value)
            .execute(pool.inner())
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok("IMAP configuration saved successfully".to_string())
}

#[tauri::command]
async fn get_imap_config(pool: State<'_, Pool<Sqlite>>) -> Result<ImapConfigStruct, String> {
    let rows = sqlx::query("SELECT key, value FROM settings WHERE key IN ('imap_host', 'imap_port', 'imap_user', 'imap_password')")
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let mut host = String::new();
    let mut port = 993; // Default IMAP port
    let mut email = String::new();
    let mut password = String::new();

    for row in rows {
        let key: String = row.get("key");
        let value: String = row.get("value");

        match key.as_str() {
            "imap_host" => host = value,
            "imap_port" => port = value.parse().unwrap_or(993),
            "imap_user" => email = value,
            "imap_password" => password = value,
            _ => {}
        }
    }

    if host.is_empty() || email.is_empty() {
        return Err("IMAP configuration not found".to_string());
    }

    Ok(ImapConfigStruct {
        host,
        port,
        email,
        password,
    })
}

pub mod db;
pub mod imap_sync;

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
            get_imap_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
