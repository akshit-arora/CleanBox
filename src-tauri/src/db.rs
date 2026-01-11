use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
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
    .expect("❌ Failed to create database schema.");

    // Create Settings Table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await
    .expect("❌ Failed to create settings table.");

    // Create Custom Rules Table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS custom_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sender_filter TEXT,       -- Exact match or pattern
            subject_filter TEXT,      -- Regex or substring
            body_filter TEXT,         -- Regex or substring
            target_view_mode TEXT NOT NULL,
            priority INTEGER DEFAULT 0
        );",
    )
    .execute(&pool)
    .await
    .expect("❌ Failed to create custom_rules table.");

    println!("✅ CleanBox Database Initialized!");
    pool
}
