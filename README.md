# 📦 CleanBox
| Reclaiming the Inbox. An offline-first, privacy-focused email client built for the modern workflow.

**CleanBox** is not just another email client. It is an **Operating System for Communication**. We believe that not all emails are created equal, and they shouldn't all look like a spreadsheet.

CleanBox leverages **Rust** and **Tauri** to categorize your email into three distinct workflows, keeping your data local, secure, and blazing fast.

## 🚀 Why CleanBox?
Most email clients treat every message the same. A newsletter, a message from your boss, and a chat with a friend all sit in the same list. **That’s broken.**

CleanBox introduces the **Tri-View Architecture**:

### 1. 📋 The Workflow View (Kanban)
**For Actionable Work.** 
Treat emails like tasks. Move emails from `Inbox` to `Doing`, `Waiting`, or `Done`.

* _Best for:_ Client requests, invoices, task assignments.

### 2. 💬 The Relationship View (Chat)
**For Human Connection.** 
View emails as a continuous chat history, sorted by person, not subject line. No more digging through threads to find what "John" sent you last month.

* _Best for:_ Team discussions, catching up with friends.

### 3. 📰 The Feed View (Digest)
**For Content Consumption.** 
Read newsletters and notifications like a social media feed. Image-first, distraction-free, and separate from your actual work.

* _Best for:_ Substack, marketing emails, notifications.

### 4. 💳 The Ledger View (Finance)
**For Your Wallet.**
Stop searching for "Order Confirmed" emails. CleanBox automatically extracts transaction details from UPI, Credit Card, and Shopping emails (Amazon, Flipkart) and presents them in a clean **Spending Table**.
* *Best for:* Bank alerts, UPI transaction history, Monthly Statements.

### 5. 🔔 The Pulse View (Notifications)
**For The Noise.**
One-time alerts like OTPs, Login verifications, and LinkedIn requests.
* *Feature:* **Auto-Expire.** Set Pulse emails to automatically delete after 7 days to keep your database light.

## 🛡️ Privacy & Philosophy
* **Local First:** Your data lives in an encrypted SQLite database on your machine.
* **Zero Trackers:** We strip pixel trackers by default. Your email activity is your business.
* **Blazing Fast:** Built on Rust, so it opens instantly and uses minimal RAM.
* **The "Clean" Promise:** No AI training on your data. No selling your metadata.

## 🛠️ Tech Stack
We are building in public! Here is the under-the-hood look:

* **Core:** Rust 🦀
* **Frontend:** Vue 3 + TypeScript
* **Framework:** Tauri v2 (for tiny binaries)
* **Database:** SQLite (via sqlx)
* **Parser:** Custom IMAP parser using imap-proto

## ⚡ Getting Started
### Prerequisites
* Rust (cargo)
* Node.js (npm or pnpm)

### Installation
```bash

# Clone the repo
git clone https://github.com/akshit-arora/Cleanbox
cd cleanbox

# Install dependencies
npm install

# Run in Development Mode
npm run tauri dev
```

## 🗺️ Roadmap (Build in Public)
Follow my journey on [X (Twitter)](https://x.com/akshitarora) as I build this from scratch!

* [x] Phase 1: Foundation
  * [x] Project Setup (Tauri + Vue)
  * [x] IMAP + SMTP Connection & Fetching
  * [x] Local SQLite Schema Design
* [x] Phase 2: The Logic
  * [x] The "Classifier Engine" (Auto-tagging emails)
  * [x] Kanban State Management
* [ ] Phase 3: The UI
  * [x] Basic UI Structure
  * [x] Settings
  * [x] Inbox View
  * [x] Draggable Kanban Board
  * [x] Chat Interface
  * [x] News Feed Layout
  * [ ] Ledger Interface
  * [ ] Pulse Interface
* [ ] Phase 4: Sync
  * [ ] E2EE Serverless Sync (The "Obsidian" Model)

## 🤝 Contributing
We welcome contributions! Whether you are a Rustacean or a Vue wizard, check out the `CONTRIBUTING.md` file (coming soon) to get started.

## 📄 License
This project is open-sourced under the MIT License.

---

*Built with ❤️ and 🦀 by [Akshit Arora](https://www.akshitarora.dev) 🇮🇳*
