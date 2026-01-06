use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Email {
    pub id: String,
    pub sender: String,
    pub subject: String,
    pub body_preview: String,
    pub view_mode: String,     // 'KANBAN', 'CHAT', 'FEED', 'LEDGER', 'PULSE'
    pub kanban_status: String, // 'INBOX', 'DONE', etc.
    // Financial Data (Extracted via Regex)
    pub amount: Option<f64>,
    pub merchant: Option<String>,
    pub received_at: String,
}

pub async fn init_db() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("cleanbox.db")
                .create_if_missing(true),
        )
        .await
        .expect("❌ Failed to connect to SQLite.");

    // Create the table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS emails (
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
    .expect("❌ Failed to create emails emails table.");

    // Create the settings table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await
    .expect("❌ Failed to create settings table.");

    println!("✅ CleanBox Database Initialized!");
    pool
}
