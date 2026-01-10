pub mod types;
pub mod strategies {
    pub mod feed;
    pub mod ledger;
    pub mod pulse;
    pub mod workflow;
}

use types::{TransactionData, ViewMode};

pub fn classify_email(
    subject: &str,
    sender: &str,
    body: &str,
) -> (ViewMode, Option<TransactionData>) {
    // 1. Check Ledger
    if let Some(result) = strategies::ledger::check(subject, sender, body) {
        return result;
    }

    // 2. Check Pulse
    if let Some(result) = strategies::pulse::check(subject, sender, body) {
        return result;
    }

    // 3. Check Feed
    if let Some(result) = strategies::feed::check(subject, sender, body) {
        return result;
    }

    // 4. Default: Workflow
    // The workflow strategy always returns Some, so we can unwrap or just rely on the fallback logic here
    // but cleaner to call the strategy if it might have logic later.
    strategies::workflow::check(subject, sender, body).unwrap_or((ViewMode::Workflow, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_ledger() {
        let (mode, _) = classify_email("Your Transaction at Starbucks", "alert@bank.com", "body");
        assert_eq!(mode, ViewMode::Ledger);

        let (mode, _) = classify_email("Payment Received", "Amazon Pay", "body");
        assert_eq!(mode, ViewMode::Ledger);
    }

    #[test]
    fn test_classify_pulse() {
        let (mode, _) = classify_email("Your Login OTP", "auth@service.com", "body");
        assert_eq!(mode, ViewMode::Pulse);

        let (mode, _) = classify_email("Verify your email", "support@app.com", "body");
        assert_eq!(mode, ViewMode::Pulse);
    }

    #[test]
    fn test_classify_feed() {
        let (mode, _) = classify_email("Weekly Newsletter", "newsletter@news.com", "body");
        assert_eq!(mode, ViewMode::Feed);

        let (mode, _) = classify_email("Marketing Promo", "marketing@shop.com", "body");
        assert_eq!(mode, ViewMode::Feed);
    }

    #[test]
    fn test_classify_workflow() {
        let (mode, _) = classify_email("Hello friend", "friend@email.com", "body");
        assert_eq!(mode, ViewMode::Workflow);
    }
}
