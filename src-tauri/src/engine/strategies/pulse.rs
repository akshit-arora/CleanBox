use crate::engine::types::{TransactionData, ViewMode};

pub fn check(
    subject: &str,
    _sender: &str,
    _body: &str,
) -> Option<(ViewMode, Option<TransactionData>)> {
    let lower_subject = subject.to_lowercase();

    if lower_subject.contains("otp")
        || lower_subject.contains("login")
        || lower_subject.contains("verify")
        || lower_subject.contains("code")
    {
        return Some((ViewMode::Pulse, None));
    }

    None
}
