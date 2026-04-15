use std::{collections::HashMap, fmt};

use anyhow::{anyhow, Result};
use clap::Parser;
use comfy_table::{Cell, Row, Table};
use io_gmail::{
    labels::list::{GmailLabelsList, GmailLabelsListResult},
    messages::get::{GmailMessageGet, GmailMessageGetResult},
    types::label::Label,
    types::message::MessageFormat,
};
use pimalaya_toolbox::terminal::printer::Printer;
use serde::Serialize;

use crate::{
    gmail::{
        account::GmailAccount, compact_flags, custom_labels, label_display_name,
        payload_has_attachment, resolve_label_id, ENVELOPE_METADATA_HEADERS,
    },
    gmail_run,
};

use super::list::{format_address_list, map_envelope_entry, message_date};

#[derive(Debug, Parser)]
pub struct GmailEnvelopeGetCommand {
    #[arg(value_name = "ID")]
    pub id: String,

    #[arg(long, short = 'l', value_name = "LABEL")]
    pub label: Option<String>,
}

impl GmailEnvelopeGetCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;
        let labels = gmail_run!(
            gmail,
            || GmailLabelsList::new(&gmail.http_auth, &gmail.user_id),
            GmailLabelsListResult
        )?
        .labels;
        let current_label_id = self
            .label
            .as_deref()
            .map(|label| {
                resolve_label_id(label, &labels)
                    .ok_or_else(|| anyhow!("Gmail label `{label}` not found"))
            })
            .transpose()?;
        let labels_by_id: HashMap<String, Label> = labels
            .into_iter()
            .map(|label| (label.id.clone(), label))
            .collect();

        let message = gmail_run!(
            gmail,
            || GmailMessageGet::new(
                &gmail.http_auth,
                &gmail.user_id,
                &self.id,
                MessageFormat::Metadata,
                &ENVELOPE_METADATA_HEADERS,
            ),
            GmailMessageGetResult
        )?;
        let entry = map_envelope_entry(&message, current_label_id.as_deref(), &labels_by_id);
        let payload = message.payload.as_ref();

        let table = EnvelopeTable {
            preset: account.table_preset,
            envelope: EnvelopeTableItems {
                id: message.id.clone(),
                message_id: payload
                    .and_then(|payload| payload.header("Message-ID"))
                    .map(ToOwned::to_owned),
                in_reply_to: payload
                    .and_then(|payload| payload.header("In-Reply-To"))
                    .map(ToOwned::to_owned),
                date: message_date(&message),
                subject: payload
                    .and_then(|payload| payload.header("Subject"))
                    .map(super::list::decode_mime),
                from: format_address_list(payload.and_then(|payload| payload.header("From"))),
                to: format_address_list(payload.and_then(|payload| payload.header("To"))),
                flags: entry.flags,
                labels: message
                    .label_ids
                    .iter()
                    .map(|label_id| label_display_name(label_id, &labels_by_id))
                    .collect(),
                custom_labels: custom_labels(
                    &message.label_ids,
                    current_label_id.as_deref(),
                    &labels_by_id,
                ),
                has_attachment: payload_has_attachment(payload),
                compact_flags: compact_flags(&message.label_ids, payload_has_attachment(payload)),
            },
        };

        printer.out(table)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EnvelopeTable {
    #[serde(skip)]
    pub preset: String,
    pub envelope: EnvelopeTableItems,
}

impl fmt::Display for EnvelopeTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_header(Row::from([Cell::new("HEADER"), Cell::new("VALUE")]));

        add_optional_row(&mut table, "ID", Some(&self.envelope.id));
        add_optional_row(
            &mut table,
            "Message ID",
            self.envelope.message_id.as_deref(),
        );
        add_optional_row(
            &mut table,
            "In Reply To",
            self.envelope.in_reply_to.as_deref(),
        );
        add_optional_row(&mut table, "Date", self.envelope.date.as_deref());
        add_optional_row(&mut table, "Subject", self.envelope.subject.as_deref());
        table.add_row(Row::from([
            Cell::new("From"),
            Cell::new(self.envelope.from.join(", ")),
        ]));
        table.add_row(Row::from([
            Cell::new("To"),
            Cell::new(self.envelope.to.join(", ")),
        ]));
        table.add_row(Row::from([
            Cell::new("Flags"),
            Cell::new(&self.envelope.flags),
        ]));
        table.add_row(Row::from([
            Cell::new("Labels"),
            Cell::new(self.envelope.labels.join(", ")),
        ]));
        table.add_row(Row::from([
            Cell::new("Custom Labels"),
            Cell::new(self.envelope.custom_labels.join(", ")),
        ]));
        table.add_row(Row::from([
            Cell::new("Has Attachment"),
            Cell::new(if self.envelope.has_attachment {
                "yes"
            } else {
                ""
            }),
        ]));

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct EnvelopeTableItems {
    pub id: String,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub date: Option<String>,
    pub subject: Option<String>,
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub flags: String,
    pub labels: Vec<String>,
    pub custom_labels: Vec<String>,
    pub has_attachment: bool,
    pub compact_flags: String,
}

fn add_optional_row(table: &mut Table, header: &str, value: Option<&str>) {
    table.add_row(Row::from([
        Cell::new(header),
        Cell::new(value.unwrap_or_default()),
    ]));
}
