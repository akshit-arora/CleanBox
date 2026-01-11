use magic_crypt::{new_magic_crypt, MagicCryptTrait};
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

const ENCRYPTION_KEY: &str = "cleanbox_secure_key_123"; // TODO: Use a better key management strategy

#[tauri::command]
async fn save_imap_config(
    pool: State<'_, Pool<Sqlite>>,
    config: ImapConfig,
) -> Result<String, String> {
    let mc = new_magic_crypt!(ENCRYPTION_KEY, 256);

    let encrypted_imap_pass = mc.encrypt_str_to_base64(&config.imap_password);
    let encrypted_smtp_pass = mc.encrypt_str_to_base64(&config.smtp_password);

    let queries = vec![
        ("imap_host", config.imap_host),
        ("imap_port", config.imap_port),
        ("imap_user", config.imap_user),
        ("imap_password", encrypted_imap_pass),
        ("smtp_host", config.smtp_host),
        ("smtp_port", config.smtp_port),
        ("smtp_user", config.smtp_user),
        ("smtp_password", encrypted_smtp_pass),
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
    let mc = new_magic_crypt!(ENCRYPTION_KEY, 256);

    for row in rows {
        let key: String = row.try_get("key").unwrap_or_default();
        let value: String = row.try_get("value").unwrap_or_default();

        match key.as_str() {
            "imap_host" => config.imap_host = value,
            "imap_port" => config.imap_port = value,
            "imap_user" => config.imap_user = value,
            "imap_password" => {
                config.imap_password = mc
                    .decrypt_base64_to_string(&value)
                    .unwrap_or_else(|_| "".to_string())
            }
            "smtp_host" => config.smtp_host = value,
            "smtp_port" => config.smtp_port = value,
            "smtp_user" => config.smtp_user = value,
            "smtp_password" => {
                config.smtp_password = mc
                    .decrypt_base64_to_string(&value)
                    .unwrap_or_else(|_| "".to_string())
            }
            _ => {}
        }
    }

    Ok(config)
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
            get_imap_config,
            imap_sync::fetch_inbox_top
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
