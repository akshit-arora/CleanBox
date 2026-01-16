use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};

// ... (imports)
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum KanbanStage {
    Inbox,
    Action,  // Doing / Today
    Waiting, // Waiting for reply
    Done,    // Archive
}

impl fmt::Display for KanbanStage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KanbanStage::Inbox => write!(f, "INBOX"),
            KanbanStage::Action => write!(f, "ACTION"),
            KanbanStage::Waiting => write!(f, "WAITING"),
            KanbanStage::Done => write!(f, "DONE"),
        }
    }
}

impl FromStr for KanbanStage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ACTION" => Ok(KanbanStage::Action),
            "WAITING" => Ok(KanbanStage::Waiting),
            "DONE" => Ok(KanbanStage::Done),
            _ => Ok(KanbanStage::Inbox), // Default to Inbox, never crash
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Email {
    pub id: String,
    pub sender: String,
    pub subject: String,
    pub body_preview: String,
    pub view_mode: String,     // 'KANBAN', 'CHAT', 'FEED', 'LEDGER', 'PULSE'
    pub kanban_status: String, // 'INBOX', 'ACTION', 'WAITING', 'DONE'
    // Financial Data (Extracted via Regex)
    pub amount: Option<f64>,
    pub merchant: Option<String>,
    pub received_at: String,
    pub body: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KanbanColumn {
    pub id: String,
    pub title: String,
    pub color: String,
}

#[derive(Serialize)]
pub struct KanbanColumnData {
    pub id: String,
    pub title: String,
    pub color: String,
    pub emails: Vec<Email>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ChatThread {
    pub sender_name: String,
    pub sender_email: String,
    pub latest_subject: String,
    pub last_message_time: String,
}

#[derive(Serialize)]
pub struct KanbanBoard {
    pub columns: Vec<KanbanColumnData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LedgerStats {
    pub total_income: f64,
    pub total_expense: f64,
    pub balance: f64,
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
            received_at TEXT NOT NULL,
            body TEXT
        );",
    )
    .execute(&pool)
    .await
    .expect("❌ Failed to create database schema.");

    // Create Index for Performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_kanban_status ON emails (kanban_status);")
        .execute(&pool)
        .await
        .expect("❌ Failed to create index on kanban_status.");

    // MIGRATION: ADD body column if not exists
    // We try to add it, if it fails (likely because it exists), we ignore the error.
    let _ = sqlx::query("ALTER TABLE emails ADD COLUMN body TEXT;")
        .execute(&pool)
        .await;

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

    // ... existing code

    println!("✅ CleanBox Database Initialized!");
    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kanban_stage_to_string() {
        assert_eq!(KanbanStage::Inbox.to_string(), "INBOX");
        assert_eq!(KanbanStage::Action.to_string(), "ACTION");
        assert_eq!(KanbanStage::Waiting.to_string(), "WAITING");
        assert_eq!(KanbanStage::Done.to_string(), "DONE");
    }

    #[test]
    fn test_kanban_stage_from_str() {
        assert_eq!(
            KanbanStage::from_str("ACTION").unwrap(),
            KanbanStage::Action
        );
        assert_eq!(
            KanbanStage::from_str("waiting").unwrap(),
            KanbanStage::Waiting
        ); // Case insensitive check
        assert_eq!(KanbanStage::from_str("DONE").unwrap(), KanbanStage::Done);
        assert_eq!(
            KanbanStage::from_str("UNKNOWN").unwrap(),
            KanbanStage::Inbox
        ); // Default fallback
        assert_eq!(KanbanStage::from_str("").unwrap(), KanbanStage::Inbox);
    }
}
