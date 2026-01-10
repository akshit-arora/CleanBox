use crate::engine::types::{TransactionData, ViewMode};

pub fn check(
    _subject: &str,
    _sender: &str,
    _body: &str,
) -> Option<(ViewMode, Option<TransactionData>)> {
    // Default fallback
    Some((ViewMode::Workflow, None))
}
