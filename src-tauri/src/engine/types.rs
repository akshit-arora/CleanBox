use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViewMode {
    Ledger,
    Pulse,
    Feed,
    Workflow,
}

impl ToString for ViewMode {
    fn to_string(&self) -> String {
        match self {
            ViewMode::Ledger => "LEDGER".to_string(),
            ViewMode::Pulse => "PULSE".to_string(),
            ViewMode::Feed => "FEED".to_string(),
            ViewMode::Workflow => "WORKFLOW".to_string(),
        }
    }
}

impl ViewMode {
    pub fn as_str(&self) -> &str {
        match self {
            ViewMode::Ledger => "LEDGER",
            ViewMode::Pulse => "PULSE",
            ViewMode::Feed => "FEED",
            ViewMode::Workflow => "WORKFLOW",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub amount: Option<f64>,
    pub merchant: Option<String>,
}
