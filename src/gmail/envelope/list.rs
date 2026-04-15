use std::{collections::HashMap, fmt};

use anyhow::{bail, Result};
use clap::Parser;
use comfy_table::{Cell, ContentArrangement, Row, Table};
use io_gmail::{
    labels::list::{GmailLabelsList, GmailLabelsListResult},
    messages::{
        get::{GmailMessageGet, GmailMessageGetResult},
        list::{GmailMessagesList, GmailMessagesListResponse, GmailMessagesListResult},
    },
    types::{
        label::Label,
        message::{Message, MessageFormat},
    },
};
use log::debug;
use mail_parser::{Addr, Address, HeaderValue, MessageParser};
use pimalaya_toolbox::terminal::printer::Printer;
use rfc2047_decoder::{Decoder, RecoverStrategy};
use serde::Serialize;

use crate::{
    gmail::{
        account::GmailAccount, compact_flags, custom_labels, parse_internal_date,
        payload_has_attachment, resolve_label_id, ENVELOPE_METADATA_HEADERS,
    },
    gmail_run,
};

#[derive(Debug, Parser)]
pub struct GmailEnvelopeListCommand {
    #[arg(long, short = 'q', value_name = "QUERY")]
    pub query: Option<String>,

    #[arg(long, short = 'l', value_name = "LABEL")]
    pub label: Option<String>,

    #[arg(long, short = 's', value_name = "N", default_value = "20")]
    pub page_size: u32,

    #[arg(long, short, value_name = "N", default_value = "1")]
    pub page: u32,
}

impl GmailEnvelopeListCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        if self.page_size == 0 {
            bail!("page-size must be greater than zero");
        }

        let mut gmail = account.new_gmail_session()?;
        let labels = gmail_run!(
            gmail,
            || GmailLabelsList::new(&gmail.http_auth, &gmail.user_id),
            GmailLabelsListResult
        )?
        .labels;

        let current_label_id = resolve_label_id(self.label.as_deref().unwrap_or("INBOX"), &labels)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Gmail label `{}` not found",
                    self.label.as_deref().unwrap_or("INBOX")
                )
            })?;
        let labels_by_id: HashMap<String, Label> = labels
            .into_iter()
            .map(|label| (label.id.clone(), label))
            .collect();
        let label_filter = vec![current_label_id.clone()];

        let page = fetch_messages_page(
            &mut gmail,
            self.query.as_deref(),
            &label_filter,
            self.page_size,
            self.page.max(1),
        )?;

        let mut envelopes = Vec::new();
        for message in page.messages {
            let response = gmail_run!(
                gmail,
                || GmailMessageGet::new(
                    &gmail.http_auth,
                    &gmail.user_id,
                    &message.id,
                    MessageFormat::Metadata,
                    &ENVELOPE_METADATA_HEADERS,
                ),
                GmailMessageGetResult
            )?;
            envelopes.push(map_envelope_entry(
                &response,
                Some(current_label_id.as_str()),
                &labels_by_id,
            ));
        }

        printer.out(EnvelopesTable {
            preset: account.table_preset,
            arrangement: account.table_arrangement,
            envelopes,
        })
    }
}

fn fetch_messages_page(
    gmail: &mut crate::gmail::account::GmailSession,
    query: Option<&str>,
    label_filter: &[String],
    page_size: u32,
    target_page: u32,
) -> Result<GmailMessagesListResponse> {
    let mut current_page = 1;
    let mut page_token = None;

    loop {
        let response = gmail_run!(
            gmail,
            || GmailMessagesList::new(
                &gmail.http_auth,
                &gmail.user_id,
                query,
                label_filter,
                Some(page_size),
                page_token.as_deref(),
                false,
            ),
            GmailMessagesListResult
        )?;

        if current_page >= target_page {
            return Ok(response);
        }

        let Some(next_page_token) = response.next_page_token else {
            bail!("page {} out of bounds", target_page);
        };

        current_page += 1;
        page_token = Some(next_page_token);
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EnvelopesTable {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    pub envelopes: Vec<EnvelopeTableEntry>,
}

impl fmt::Display for EnvelopesTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("FLAGS"),
                Cell::new("SUBJECT"),
                Cell::new("FROM"),
                Cell::new("DATE"),
            ]));

        for entry in &self.envelopes {
            let mut row = Row::new();
            row.max_height(1);
            row.add_cell(Cell::new(&entry.id));
            row.add_cell(Cell::new(&entry.flags));
            row.add_cell(Cell::new(&entry.subject));
            row.add_cell(Cell::new(&entry.from));
            row.add_cell(Cell::new(&entry.date));
            table.add_row(row);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct EnvelopeTableEntry {
    pub id: String,
    pub flags: String,
    pub date: String,
    pub from: String,
    pub subject: String,
}

pub(crate) fn map_envelope_entry(
    message: &Message,
    current_label_id: Option<&str>,
    labels_by_id: &HashMap<String, Label>,
) -> EnvelopeTableEntry {
    let payload = message.payload.as_ref();
    let mut flags = compact_flags(&message.label_ids, payload_has_attachment(payload));

    if !custom_labels(&message.label_ids, current_label_id, labels_by_id).is_empty() {
        flags.push('+');
    }

    EnvelopeTableEntry {
        id: message.id.clone(),
        flags,
        date: message_date(message).unwrap_or_default(),
        from: format_address_list_short(payload.and_then(|payload| payload.header("From"))),
        subject: payload
            .and_then(|payload| payload.header("Subject"))
            .map(decode_mime)
            .unwrap_or_default(),
    }
}

pub(crate) fn decode_mime(value: &str) -> String {
    let decoder = Decoder::new().too_long_encoded_word_strategy(RecoverStrategy::Decode);
    match decoder.decode(value.as_bytes()) {
        Ok(decoded) => decoded,
        Err(err) => {
            debug!("cannot decode rfc2047 string `{value}`: {err}");
            value.to_string()
        }
    }
}

pub(crate) fn format_address_list_short(value: Option<&str>) -> String {
    parse_address_list("From", value, true).join(", ")
}

pub(crate) fn format_address_list(value: Option<&str>) -> Vec<String> {
    parse_address_list("From", value, false)
}

pub(crate) fn message_date(message: &Message) -> Option<String> {
    let header_date = message
        .payload
        .as_ref()
        .and_then(|payload| payload.header("Date"))
        .map(ToOwned::to_owned)
        .filter(|date| !date.trim().is_empty());

    header_date.or_else(|| {
        message
            .internal_date
            .as_deref()
            .and_then(parse_internal_date)
    })
}

fn parse_address_list(header_name: &str, value: Option<&str>, short: bool) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };

    let raw = format!("{header_name}: {value}\r\n\r\n");
    let Some(message) = MessageParser::new().parse_headers(raw.as_bytes()) else {
        return vec![decode_mime(value)];
    };

    let Some(header) = message.headers().first() else {
        return vec![decode_mime(value)];
    };

    match header.value() {
        HeaderValue::Address(Address::List(addrs)) => addrs
            .iter()
            .map(|addr| format_addr(addr, short))
            .filter(|addr| !addr.is_empty())
            .collect(),
        HeaderValue::Address(Address::Group(groups)) => groups
            .iter()
            .flat_map(|group| group.addresses.iter())
            .map(|addr| format_addr(addr, short))
            .filter(|addr| !addr.is_empty())
            .collect(),
        _ => vec![decode_mime(value)],
    }
}

fn format_addr(addr: &Addr<'_>, short: bool) -> String {
    let email = addr
        .address
        .as_deref()
        .map(str::trim)
        .filter(|email| !email.is_empty())
        .unwrap_or_default()
        .to_string();
    let name = addr
        .name
        .as_deref()
        .map(decode_mime)
        .filter(|name| !name.trim().is_empty());

    match (short, name, email.is_empty()) {
        (true, Some(name), _) => name,
        (false, Some(name), false) => format!("{name} <{email}>"),
        (false, Some(name), true) => name,
        _ => email,
    }
}
