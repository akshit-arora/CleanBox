use crate::engine::types::{TransactionData, ViewMode};

pub fn check(
    _subject: &str,
    sender: &str,
    _body: &str,
) -> Option<(ViewMode, Option<TransactionData>)> {
    let lower_sender = sender.to_lowercase();

    if lower_sender.contains("newsletter")
        || lower_sender.contains("no-reply")
        || lower_sender.contains("marketing")
        || lower_sender.contains("promo")
    {
        return Some((ViewMode::Feed, None));
    }

    None
}
