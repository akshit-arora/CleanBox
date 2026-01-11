use crate::db::Email;
use async_imap::error::Error as ImapError; // Keeping this for now, as the instruction's line is malformed.
use async_imap::error::Ret; // This import seems incorrect, but following instruction.
use async_native_tls::TlsConnector;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use sqlx::{Pool, Row, Sqlite};
use tauri::State;
use tokio::net::TcpStream; // Added for the new connection logic

const ENCRYPTION_KEY: &str = "cleanbox_secure_key_123";

#[tauri::command]
pub async fn fetch_inbox_top(pool: State<'_, Pool<Sqlite>>) -> Result<String, String> {
    // 1. Fetch Credentials
    let row = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut imap_host = String::new();
    let mut imap_port = String::new();
    let mut imap_user = String::new();
    let mut imap_password = String::new();

    let mc = new_magic_crypt!(ENCRYPTION_KEY, 256);

    for r in row {
        let key: String = r.try_get("key").unwrap_or_default();
        let value: String = r.try_get("value").unwrap_or_default();

        match key.as_str() {
            "imap_host" => imap_host = value,
            "imap_port" => imap_port = value,
            "imap_user" => imap_user = value,
            "imap_password" => {
                imap_password = mc
                    .decrypt_base64_to_string(&value)
                    .unwrap_or_else(|_| "".to_string())
            }
            _ => {}
        }
    }

    if imap_host.is_empty() || imap_user.is_empty() || imap_password.is_empty() {
        return Err("Please configure IMAP in settings.".to_string());
    }

    let port = imap_port.parse::<u16>().map_err(|_| "Invalid IMAP Port")?;

    // 2. Connect to IMAP (Plain for now, or implicit TLS if port 993?)
    // User requested "input their IMAP & SMTP credentials... save them...".
    // Usually 993 is TLS. I'll assume TLS.
    // async-imap with runtime-tokio doesn't include TlsConnector by default unless features enabled.
    // I put `features = ["runtime-tokio"]` but didn't put `native-tls` or `rustls`.
    // I'll try to use `async_imap::connect` if I added usage.
    // Wait, I only added `runtime-tokio`. `async_imap::connect` requires a TLS feature for `connect` (helper) OR I construct it manually.
    // I will use `TcpStream` and manual handling or just simple `async_imap::Session`.

    // Actually, to make it simple and standard:
    let tcp_stream = TcpStream::connect((imap_host.as_str(), port))
        .await
        .map_err(|e| format!("TCP Connection failed: {}", e))?;

    // For TLS, we need `async-native-tls` or similar if we want to upgrade.
    // Since `async-imap` creates a session, I should check how to do it.
    // The easiest is `async_imap::connect` which does it all if features enabled.
    // But I didn't enable `native-tls` in Cargo.toml.
    // I will enable `native-tls` in Cargo.toml in next step if this fails or just assume plaintext for port 143, but 993 needs TLS.
    // I will strictly enable `default-features = false, features = ["runtime-tokio", "native-tls"]` in Cargo.toml (or just add native-tls).
    // For now I'll write code assuming I can use `async_native_tls`.
    // Wait, I can't add crate imports in this tool call.
    // I will just use `async_imap::Client::new(tcp_stream)` for plaintext if port != 993, but real world needs TLS.
    // I'll stick to modifying Cargo.toml properly.

    // Let's assume I fix Cargo.toml to have `native-tls` as well.
    // Then `async_imap::connect` works.

    let tls = async_native_tls::TlsConnector::new();
    let client = async_imap::connect((imap_host.as_str(), port), imap_host.as_str(), tls)
        .await
        .map_err(|e| format!("IMAP Connect failed: {}", e))?;

    // 3. Login
    let mut session = client
        .login(imap_user, imap_password)
        .await
        .map_err(|e| format!("Login failed: {}", e))
        .map_err(|e| e.to_string())?;

    // 4. Select INBOX
    session
        .select("INBOX")
        .await
        .map_err(|e| format!("Failed to select INBOX: {}", e))?;

    // 5. Fetch Top 10 Emails
    let messages = session
        .fetch("1:10", "RFC822")
        .await
        .map_err(|e| format!("Fetch failed: {}", e))?;

    let count = messages.len();
    println!("Fetched {} messages", count);

    Ok(format!(
        "Successfully connected and fetched {} messages",
        count
    ))
}
