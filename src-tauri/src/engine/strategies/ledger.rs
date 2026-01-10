use crate::engine::types::{TransactionData, ViewMode};

pub fn check(
    subject: &str,
    sender: &str,
    _body: &str,
) -> Option<(ViewMode, Option<TransactionData>)> {
    let lower_subject = subject.to_lowercase();
    let lower_sender = sender.to_lowercase();

    let financial_keywords = ["transaction", "txn", "inr", "rs./", "payment", "spent"];
    let banks = [
        "sbi",
        "icici",
        "kotak",
        "hdfc",
        "amazon pay",
        "paytm",
        "gpay",
    ];

    let is_financial = financial_keywords.iter().any(|k| lower_subject.contains(k))
        || banks.iter().any(|b| lower_sender.contains(b));

    if is_financial {
        // TODO: Extract actual amount and merchant using regex later
        // For now, we return empty transaction data
        return Some((
            ViewMode::Ledger,
            Some(TransactionData {
                amount: None,
                merchant: None,
            }),
        ));
    }

    None
}
