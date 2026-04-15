use std::collections::BTreeSet;

use anyhow::{bail, Result};
use io_gmail::types::label::Label;

use crate::gmail::resolve_label_id;

pub mod add;
pub mod command;
pub mod remove;

enum FlagClass {
    System(&'static str),
    SystemInverse(&'static str),
    Custom(String),
}

fn classify_flag(flag: &str) -> Result<FlagClass> {
    match flag.to_ascii_lowercase().as_str() {
        "seen" | "\\seen" => Ok(FlagClass::SystemInverse("UNREAD")),
        "unseen" | "\\unseen" | "unread" => Ok(FlagClass::System("UNREAD")),
        "flagged" | "\\flagged" | "starred" => Ok(FlagClass::System("STARRED")),
        "important" | "\\important" => Ok(FlagClass::System("IMPORTANT")),
        "draft" | "\\draft" => Ok(FlagClass::System("DRAFT")),
        "deleted" | "\\deleted" | "trash" => Ok(FlagClass::System("TRASH")),
        "answered" | "\\answered" => bail!("Answered is not supported on Gmail"),
        other => Ok(FlagClass::Custom(other.to_string())),
    }
}

pub(crate) fn build_label_updates(
    flags: &[String],
    labels: &[Label],
    remove_mode: bool,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut add = BTreeSet::new();
    let mut remove = BTreeSet::new();

    for flag in flags {
        match classify_flag(flag)? {
            FlagClass::System(label_id) => {
                if remove_mode {
                    remove.insert(label_id.to_string());
                } else {
                    add.insert(label_id.to_string());
                }
            }
            FlagClass::SystemInverse(label_id) => {
                if remove_mode {
                    add.insert(label_id.to_string());
                } else {
                    remove.insert(label_id.to_string());
                }
            }
            FlagClass::Custom(label) => {
                let Some(label_id) = resolve_label_id(&label, labels) else {
                    bail!("Gmail label `{label}` not found");
                };

                if remove_mode {
                    remove.insert(label_id);
                } else {
                    add.insert(label_id);
                }
            }
        }
    }

    Ok((add.into_iter().collect(), remove.into_iter().collect()))
}
