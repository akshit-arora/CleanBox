use crate::db::Email;
use futures::TryStreamExt;
use mail_parser::{HeaderValue, Message};
use native_tls::TlsConnector as NativeTlsConnector;
use sqlx::{Pool, Row, Sqlite};
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;

pub async fn fetch_inbox_top(pool: &Pool<Sqlite>) -> Result<Vec<Email>, String> {
    // 1. Fetch credentials from DB
    let rows = sqlx::query("SELECT key, value FROM settings WHERE key IN ('imap_host', 'imap_port', 'imap_user', 'imap_password')")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let mut host = String::new();
    let mut port = 993;
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

    if host.is_empty() || email.is_empty() || password.is_empty() {
        return Err("Please configure IMAP in settings.".to_string());
    }

    // 2. Connect to IMAP (Manual TCP + TLS)
    let tcp = TcpStream::connect((host.as_str(), port))
        .await
        .map_err(|e| format!("Failed to connect to IMAP server: {}", e))?;

    let native_tls =
        NativeTlsConnector::new().map_err(|e| format!("Failed to create TLS connector: {}", e))?;
    let tls = TlsConnector::from(native_tls);

    let tls_stream = tls
        .connect(host.as_str(), tcp)
        .await
        .map_err(|e| format!("TLS handshake failed: {}", e))?;

    let client = async_imap::Client::new(tls_stream);

    // 3. Login
    let mut session = client
        .login(email, password)
        .await
        .map_err(|e| format!("Failed to login to IMAP: {}", e.0))?;

    // 4. Select INBOX
    session
        .select("INBOX")
        .await
        .map_err(|e| format!("Failed to select INBOX: {}", e))?;

    // 5. Fetch emails
    let messages: Vec<_> = session
        .fetch("1:10", "RFC822")
        .await
        .map_err(|e| format!("Failed to fetch emails: {}", e))?
        .try_collect()
        .await
        .map_err(|e| format!("Failed to collect emails: {}", e))?;

    let mut emails = Vec::new();

    for message in messages.iter() {
        if let Some(body) = message.body() {
            if let Some(parsed) = Message::parse(body) {
                let subject = parsed.subject().unwrap_or_default().to_string();

                let sender = match parsed.from() {
                    HeaderValue::Address(addr) => {
                        addr.address.as_deref().unwrap_or_default().to_string()
                    }
                    HeaderValue::Text(t) => t.to_string(),
                    _ => "Unknown Sender".to_string(),
                };

                let body_preview: String = parsed
                    .body_text(0)
                    .unwrap_or_default()
                    .chars()
                    .take(100)
                    .collect();

                // Classify the email
                let body_content = parsed.body_text(0).unwrap_or_default();
                let (view_mode, transaction_data) =
                    crate::engine::classify_email(&subject, &sender, &body_content);

                let (amount, merchant) = if let Some(data) = transaction_data {
                    (data.amount, data.merchant)
                } else {
                    (None, None)
                };

                let email_obj = Email {
                    id: message.message.to_string(), // Using sequence number as ID for now
                    sender: sender.clone(),
                    subject: subject.clone(),
                    body_preview: body_preview.clone(),
                    view_mode: view_mode.to_string(),
                    kanban_status: "INBOX".to_string(),
                    amount,
                    merchant: merchant.clone(),
                    received_at: chrono::Local::now().to_rfc3339(), // Placeholder
                };

                emails.push(email_obj);

                // Insert into DB
                let _ = sqlx::query(
                    "INSERT INTO emails (id, sender, subject, body_preview, view_mode, kanban_status, amount, merchant, received_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(id) DO UPDATE SET view_mode = excluded.view_mode, amount = excluded.amount, merchant = excluded.merchant"
                )
                .bind(message.message.to_string())
                .bind(sender)
                .bind(subject)
                .bind(body_preview)
                .bind(view_mode.to_string())
                .bind("INBOX")
                .bind(amount)
                .bind(merchant)
                .bind(chrono::Local::now().to_rfc3339())
                .execute(pool)
                .await;
            }
        }
    }

    Ok(emails)
}
