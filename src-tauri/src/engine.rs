use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ViewMode {
    Ledger, // Financials
    Pulse,  // OTPs, Alerts
    Feed,   // Newsletters
    Workflow,
}

impl ViewMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewMode::Ledger => "LEDGER",
            ViewMode::Pulse => "PULSE",
            ViewMode::Feed => "FEED",
            ViewMode::Workflow => "WORKFLOW",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "LEDGER" => ViewMode::Ledger,
            "PULSE" => ViewMode::Pulse,
            "FEED" => ViewMode::Feed,
            _ => ViewMode::Workflow,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionData {
    pub amount: Option<f64>,
    pub merchant: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct CustomRule {
    pub id: i64,
    pub name: String,
    pub sender_filter: Option<String>,
    pub subject_filter: Option<String>,
    pub body_filter: Option<String>,
    pub target_view_mode: String,
    pub priority: Option<i64>,
}

pub fn classify_email(
    subject: &str,
    sender: &str,
    body: &str,
    rules: &[CustomRule],
) -> (ViewMode, Option<TransactionData>) {
    // 0. Custom Rules (Priority)
    for rule in rules {
        let mut match_score = 0;
        let mut criteria_count = 0;

        // Sender
        if let Some(ref filter) = rule.sender_filter {
            criteria_count += 1;
            if sender.to_lowercase().contains(&filter.to_lowercase()) {
                match_score += 1;
            }
        }

        // Subject
        if let Some(ref filter) = rule.subject_filter {
            criteria_count += 1;
            let matches = if let Ok(re) = Regex::new(filter) {
                re.is_match(subject)
            } else {
                subject.to_lowercase().contains(&filter.to_lowercase())
            };
            if matches {
                match_score += 1;
            }
        }

        // Body
        if let Some(ref filter) = rule.body_filter {
            criteria_count += 1;
            let matches = if let Ok(re) = Regex::new(filter) {
                re.is_match(body)
            } else {
                body.to_lowercase().contains(&filter.to_lowercase())
            };
            if matches {
                match_score += 1;
            }
        }

        // Rule matches if ALL defined criteria pass
        if criteria_count > 0 && match_score == criteria_count {
            let mode = ViewMode::from_str(&rule.target_view_mode);
            let transaction_data = if mode == ViewMode::Ledger {
                extract_transaction_data(sender, body)
            } else {
                None
            };
            return (mode, transaction_data);
        }
    }

    // Default Rules
    classify_email_default(subject, sender, body)
}

fn classify_email_default(
    subject: &str,
    sender: &str,
    body: &str,
) -> (ViewMode, Option<TransactionData>) {
    // 1. LEDGER Rules
    let ledger_senders = ["HDFC", "SBI", "Paytm", "Kotak", "ICICI"];
    let is_ledger_sender = ledger_senders
        .iter()
        .any(|s| sender.to_uppercase().contains(&s.to_uppercase()));

    let amount_regex = Regex::new(r"(?i)(?:Rs\.?|INR)\s*([\d,]+(?:\.\d{2})?)").unwrap();

    if is_ledger_sender || amount_regex.is_match(body) {
        let tx_data = extract_transaction_data(sender, body);
        return (ViewMode::Ledger, tx_data);
    }

    // 2. PULSE Rules
    let pulse_keywords = ["OTP", "Login", "Verify"];
    if pulse_keywords.iter().any(|k| subject.contains(k)) {
        return (ViewMode::Pulse, None);
    }

    // 3. FEED Rules
    let feed_senders = ["newsletter", "no-reply", "marketing"];
    if feed_senders
        .iter()
        .any(|s| sender.to_lowercase().contains(s))
    {
        return (ViewMode::Feed, None);
    }

    (ViewMode::Workflow, None)
}

fn extract_transaction_data(sender: &str, body: &str) -> Option<TransactionData> {
    // Regex matches:
    // 1. Symbol/Code (Start) + Amount: "$ 100", "Rs. 100", "USD 100", "EUR 100"
    // 2. Amount + Symbol/Code (End): "100 USD", "100 €"
    // Supported: Rs, INR, USD, EUR, GBP, CAD, AUD, JPY, CNY, $ £ € ¥ ₹
    let currency_pattern = r"(?i)(?:Rs\.?|INR|USD|EUR|GBP|CAD|AUD|JPY|CNY|CHF|RUB|KRW|[$£€¥₹])";

    // Pattern 1: Currency then Amount
    let re_prefix = Regex::new(&format!(r"{}\s*([\d,]+(?:\.\d{{2}})?)", currency_pattern)).unwrap();

    // Pattern 2: Amount then Currency
    let re_suffix = Regex::new(&format!(r"([\d,]+(?:\.\d{{2}})?)\s*{}", currency_pattern)).unwrap();

    let mut amount = None;

    // Try Prefix First
    if let Some(caps) = re_prefix.captures(body) {
        if let Some(m) = caps.get(1) {
            let amount_str = m.as_str().replace(",", "");
            amount = amount_str.parse::<f64>().ok();
        }
    }
    // If not found, try Suffix
    else if let Some(caps) = re_suffix.captures(body) {
        if let Some(m) = caps.get(1) {
            let amount_str = m.as_str().replace(",", "");
            amount = amount_str.parse::<f64>().ok();
        }
    }

    Some(TransactionData {
        amount,
        merchant: Some(sender.to_string()),
    })
}

pub fn generate_smart_rule(
    target_email: &crate::db::Email,
    related_emails: &[crate::db::Email],
    target_view_mode: String,
) -> CustomRule {
    // 1. Sender Strategy
    let sender_filter = Some(target_email.sender.clone());

    // 2. Subject Strategy
    let mut subject_filter = None;

    if !related_emails.is_empty() {
        let subjects: Vec<&str> = related_emails.iter().map(|e| e.subject.as_str()).collect();
        if let Some(prefix) = find_common_prefix(&subjects) {
            // Trim whitespace and special chars from end
            let prefix_clean = prefix
                .trim()
                .trim_end_matches(|c: char| !c.is_alphanumeric());
            if prefix_clean.len() > 3 {
                subject_filter = Some(prefix_clean.to_string());
            }
        }
    }

    CustomRule {
        id: 0,
        name: format!("Auto-generated rule for {}", target_view_mode),
        sender_filter,
        subject_filter,
        body_filter: None,
        target_view_mode,
        priority: Some(10),
    }
}

fn find_common_prefix(strings: &[&str]) -> Option<String> {
    if strings.is_empty() {
        return None;
    }
    let first = strings[0];
    let mut len = first.len();
    // Compare first string with all others
    for s in &strings[1..] {
        while !s.starts_with(&first[..len]) {
            if len == 0 {
                return None;
            }
            len -= 1;
        }
    }
    if len > 0 {
        Some(first[..len].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Email;

    fn make_email(subject: &str, sender: &str) -> Email {
        Email {
            id: "1".to_string(),
            sender: sender.to_string(),
            subject: subject.to_string(),
            body_preview: "".to_string(),
            view_mode: "WORKFLOW".to_string(),
            kanban_status: "INBOX".to_string(),
            amount: None,
            merchant: None,
            received_at: "2023-01-01".to_string(),
        }
    }

    #[test]
    fn test_generate_rule_common_prefix() {
        let e1 = make_email("JIRA: Bug 1", "jira@company.com");
        let e2 = make_email("JIRA: Bug 2", "jira@company.com");
        let e3 = make_email("JIRA: Feature", "jira@company.com");

        let rule = generate_smart_rule(&e1, &[e1.clone(), e2, e3], "WORKFLOW".to_string());

        assert_eq!(rule.sender_filter, Some("jira@company.com".to_string()));
        // find_common_prefix returns "JIRA: ", trim cleans it maybe?
        // trim_end_matches non-alphanumeric will strip ": ". So "JIRA".
        assert_eq!(rule.subject_filter, Some("JIRA".to_string()));
    }

    #[test]
    fn test_custom_rule_overrides_default() {
        let rules = vec![CustomRule {
            id: 1,
            name: "Test Rule".to_string(),
            sender_filter: Some("Google".to_string()),
            subject_filter: None,
            body_filter: None,
            target_view_mode: "LEDGER".to_string(),
            priority: Some(10),
        }];

        let (mode, _) = classify_email("Login Alert", "Google", "Body", &rules);
        assert_eq!(mode, ViewMode::Ledger);
    }

    #[test]
    fn test_currency_extraction() {
        // Rs prefix
        let (_, data) = classify_email("Statement", "Bank", "Paid Rs. 1,500.00 to Merch", &[]);
        assert_eq!(data.unwrap().amount, Some(1500.00));

        // USD prefix
        let (_, _data) = classify_email("Receipt", "Paypal", "You sent $20.50 USD", &[]);
        // classify_email only calls extract if it hits LEDGER rules.
        // But "Paypal" is not in LEDGER senders list yet.
        // We need to verify extract_transaction_data logic directly OR use a LEDGER sender.

        let tx = extract_transaction_data("Test", "Total: $123.45");
        assert_eq!(tx.unwrap().amount, Some(123.45));

        let tx = extract_transaction_data("Test", "Total: 100 EUR");
        assert_eq!(tx.unwrap().amount, Some(100.00));

        let tx = extract_transaction_data("Test", "Total: 50.00 GBP");
        assert_eq!(tx.unwrap().amount, Some(50.00));

        let tx = extract_transaction_data("Test", "Cost: ¥5,000");
        assert_eq!(tx.unwrap().amount, Some(5000.00));
    }
}
