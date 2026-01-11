use crate::secure_store;
use futures_util::TryStreamExt;
use native_tls::TlsConnector;
use sqlx::{Pool, Row, Sqlite};
use tokio_native_tls::TlsConnector as TokioTlsConnector;

use tauri::State;
use tokio::net::TcpStream;

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

    for r in row {
        let key: String = r.try_get("key").unwrap_or_default();
        let value: String = r.try_get("value").unwrap_or_default();

        match key.as_str() {
            "imap_host" => imap_host = value,
            "imap_port" => imap_port = value,
            "imap_user" => imap_user = value,
            _ => {}
        }
    }

    let imap_password = secure_store::get_secret("imap_password").unwrap_or_default();

    if imap_host.is_empty() || imap_user.is_empty() || imap_password.is_empty() {
        return Err("Please configure IMAP in settings.".to_string());
    }

    let port = imap_port.parse::<u16>().map_err(|_| "Invalid IMAP Port")?;

    // 2. Connect to IMAP (Plain for now, or implicit TLS if port 993?)
    let tcp_stream = TcpStream::connect((imap_host.as_str(), port))
        .await
        .map_err(|e| format!("TCP Connection failed: {}", e))?;

    let tls_connector = TlsConnector::new().map_err(|e| format!("TLS Init failed: {}", e))?;
    let tls = TokioTlsConnector::from(tls_connector);
    let encrypted_stream = tls
        .connect(imap_host.as_str(), tcp_stream)
        .await
        .map_err(|e| format!("TLS Handshake failed: {}", e))?;

    let client = async_imap::Client::new(encrypted_stream);

    // 3. Login
    let mut session = client
        .login(imap_user, imap_password)
        .await
        .map_err(|e| format!("Login failed: {}", e.0))
        .map_err(|e| e.to_string())?;

    // 4. Select INBOX
    session
        .select("INBOX")
        .await
        .map_err(|e| format!("Failed to select INBOX: {}", e))?;

    // 5. Fetch Top 10 Emails
    let messages: Vec<_> = session
        .fetch("1:10", "RFC822")
        .await
        .map_err(|e| format!("Fetch failed: {}", e))?
        .try_collect()
        .await
        .map_err(|e| format!("Collect failed: {}", e))?;

    let count = messages.len();
    println!("Fetched {} messages", count);

    Ok(format!(
        "Successfully connected and fetched {} messages",
        count
    ))
}
