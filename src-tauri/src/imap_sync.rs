use crate::engine;
use crate::secure_store;
use futures_util::TryStreamExt;
use mail_parser::Message;
use native_tls::TlsConnector;
use sqlx::{Pool, Row, Sqlite};
use std::borrow::Cow;
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

    // 1.5 Fetch Custom Rules
    let custom_rules = sqlx::query_as::<_, engine::CustomRule>(
        "SELECT * FROM custom_rules ORDER BY priority DESC",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| format!("Failed to fetch custom rules: {}", e))?;

    let port = imap_port.parse::<u16>().map_err(|_| "Invalid IMAP Port")?;

    // 2. Connect to IMAP
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

    // 6. Process & Insert Emails
    for msg in messages {
        if let Some(body) = msg.body() {
            if let Some(parsed) = Message::parse(body) {
                let subject = parsed.subject().unwrap_or("(No Subject)");

                let sender = match parsed.from() {
                    mail_parser::HeaderValue::Address(addr) => addr
                        .name
                        .as_deref()
                        .or(addr.address.as_deref())
                        .unwrap_or("Unknown"),
                    val => val.as_text_ref().unwrap_or("Unknown"),
                };

                let body_text = parsed.body_text(0).unwrap_or(Cow::Borrowed("")).to_string();
                let body_preview = body_text.lines().take(2).collect::<Vec<_>>().join(" ");

                // Use Message-ID as ID, or generate a fallback
                let message_id = parsed.message_id().unwrap_or("").to_string();
                let id = if message_id.is_empty() {
                    format!(
                        "{}-{}",
                        sender,
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                    )
                } else {
                    message_id
                };

                // Classify with Rules
                let (view_mode, tx_data) =
                    engine::classify_email(subject, sender, &body_text, &custom_rules);

                let (amount, merchant) = if let Some(tx) = tx_data {
                    (tx.amount, tx.merchant)
                } else {
                    (None, None)
                };

                let received_at = chrono::Local::now().to_rfc3339();

                sqlx::query(
                    "INSERT INTO emails (id, sender, subject, body_preview, view_mode, kanban_status, amount, merchant, received_at) 
                     VALUES (?, ?, ?, ?, ?, 'INBOX', ?, ?, ?)
                     ON CONFLICT(id) DO UPDATE SET view_mode = excluded.view_mode, amount = excluded.amount, merchant = excluded.merchant"
                )
                .bind(id)
                .bind(sender)
                .bind(subject)
                .bind(body_preview)
                .bind(view_mode.as_str())
                .bind(amount)
                .bind(merchant)
                .bind(received_at)
                .execute(&*pool)
                .await
                .map_err(|e| format!("DB Insert failed: {}", e))?;
            }
        }
    }

    Ok(format!(
        "Successfully connected and fetched/processed {} messages",
        count
    ))
}
