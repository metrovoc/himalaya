use std::fmt;

use anyhow::{bail, Result};
use clap::Parser;
use io_gmail::{
    messages::get::{GmailMessageGet, GmailMessageGetResult},
    types::message::{decode_raw, MessageFormat},
};
use pimalaya_toolbox::terminal::printer::Printer;
use serde::Serialize;

use crate::{gmail::account::GmailAccount, gmail_run};

#[derive(Debug, Parser)]
pub struct GmailMessageGetCommand {
    #[arg(value_name = "ID", required = true)]
    pub ids: Vec<String>,
}

impl GmailMessageGetCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;
        let mut messages = Vec::new();

        for id in self.ids {
            let response = gmail_run!(
                gmail,
                || GmailMessageGet::new(
                    &gmail.http_auth,
                    &gmail.user_id,
                    &id,
                    MessageFormat::Raw,
                    &[]
                ),
                GmailMessageGetResult
            )?;
            let Some(raw) = response.raw.as_deref() else {
                bail!("Message `{}` does not contain RAW content", response.id);
            };
            messages.push(RawMessage {
                id: response.id,
                raw: String::from_utf8_lossy(&decode_raw(raw)?).into_owned(),
            });
        }

        printer.out(RawMessagesView { messages })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RawMessagesView {
    pub messages: Vec<RawMessage>,
}

impl fmt::Display for RawMessagesView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, message) in self.messages.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }
            if self.messages.len() > 1 {
                writeln!(f, "# {}", message.id)?;
            }
            write!(f, "{}", message.raw.trim_end())?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RawMessage {
    pub id: String,
    pub raw: String,
}
