# ARCHITECTURE.md

## Project Identity
**CleanBox** is an offline-first email client that separates "Work" (Kanban), "People" (Chat), "Reading" (Feed), and "Money" (Ledger).

## Data Model
- **Database:** SQLite (local file `cleanbox.db`)
- **Privacy:** 0 Trackers. 0 Cloud Storage.

## The 5 Views
1. **Workflow (Kanban):** For actionable emails.
2. **Relationship (Chat):** For human conversations.
3. **Consumption (Feed):** For newsletters/marketing.
4. **Ledger (Finance):** Parses UPI/Credit Card emails into a table.
5. **Pulse (Notifications):** Auto-deleting OTPs and alerts.

## Key Rust Crates
- `sqlx` (SQLite)
- `regex` (For parsing Indian bank formats like HDFC/SBI)
- `async-imap` (Fetcher)