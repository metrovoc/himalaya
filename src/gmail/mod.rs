use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use io_gmail::types::{label::Label, message::MessagePayload};

pub mod account;
pub mod command;
pub mod envelope;
pub mod flag;
pub mod label;
pub mod message;

pub(crate) const ENVELOPE_METADATA_HEADERS: [&str; 6] =
    ["Message-ID", "In-Reply-To", "Date", "From", "To", "Subject"];

pub(crate) fn find_system_label_id(label: &str) -> Option<&'static str> {
    let label = label.trim().to_ascii_lowercase();

    match label.as_str() {
        "inbox" => Some("INBOX"),
        "sent" | "sent mail" | "[gmail]/sent mail" | "[google mail]/sent mail" => Some("SENT"),
        "draft" | "drafts" | "[gmail]/drafts" | "[google mail]/drafts" => Some("DRAFT"),
        "trash" | "[gmail]/trash" | "[google mail]/trash" => Some("TRASH"),
        "junk" | "spam" | "[gmail]/spam" | "[google mail]/spam" => Some("SPAM"),
        _ => None,
    }
}

pub(crate) fn resolve_label_id(label: &str, labels: &[Label]) -> Option<String> {
    if let Some(system_id) = find_system_label_id(label) {
        return Some(system_id.to_string());
    }

    labels
        .iter()
        .find(|entry| entry.id == label || entry.name.eq_ignore_ascii_case(label))
        .map(|entry| entry.id.clone())
}

pub(crate) fn label_display_name(label_id: &str, labels_by_id: &HashMap<String, Label>) -> String {
    labels_by_id
        .get(label_id)
        .map(|label| {
            if label.name.trim().is_empty() {
                label.id.clone()
            } else {
                label.name.clone()
            }
        })
        .unwrap_or_else(|| label_id.to_string())
}

pub(crate) fn should_skip_current_folder_label(
    current_label_id: Option<&str>,
    label_id: &str,
) -> bool {
    current_label_id
        .filter(|current| *current == label_id)
        .filter(|current| !matches!(*current, "DRAFT" | "TRASH"))
        .is_some()
}

pub(crate) fn is_reserved_gmail_label(label_id: &str) -> bool {
    matches!(
        label_id,
        "INBOX" | "SENT" | "SPAM" | "UNREAD" | "STARRED" | "IMPORTANT" | "CHAT"
    ) || label_id.starts_with("CATEGORY_")
}

pub(crate) fn payload_has_attachment(payload: Option<&MessagePayload>) -> bool {
    let Some(payload) = payload else {
        return false;
    };

    if !payload.filename.trim().is_empty() {
        return true;
    }

    payload
        .parts
        .iter()
        .any(|part| payload_has_attachment(Some(part)))
}

pub(crate) fn compact_flags(label_ids: &[String], has_attachment: bool) -> String {
    let mut flags = String::new();

    if label_ids.iter().any(|label| label == "UNREAD") {
        flags.push('U');
    }
    if label_ids.iter().any(|label| label == "STARRED") {
        flags.push('F');
    }
    if label_ids.iter().any(|label| label == "IMPORTANT") {
        flags.push('I');
    }
    if label_ids.iter().any(|label| label == "DRAFT") {
        flags.push('D');
    }
    if label_ids.iter().any(|label| label == "TRASH") {
        flags.push('T');
    }
    if has_attachment {
        flags.push('A');
    }

    flags
}

pub(crate) fn custom_labels(
    label_ids: &[String],
    current_label_id: Option<&str>,
    labels_by_id: &HashMap<String, Label>,
) -> Vec<String> {
    label_ids
        .iter()
        .filter(|label_id| !should_skip_current_folder_label(current_label_id, label_id))
        .filter(|label_id| !is_reserved_gmail_label(label_id))
        .map(|label_id| label_display_name(label_id, labels_by_id))
        .collect()
}

pub(crate) fn parse_internal_date(value: &str) -> Option<String> {
    let timestamp = value.parse::<i64>().ok()?;
    let date = Utc.timestamp_millis_opt(timestamp).single()?;
    Some(date.to_rfc3339())
}
