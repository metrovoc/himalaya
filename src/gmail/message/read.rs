use std::fmt;

use anyhow::{bail, Result};
use clap::Parser;
use io_gmail::{
    messages::get::{GmailMessageGet, GmailMessageGetResult},
    types::message::{decode_raw, MessageFormat},
};
use mail_parser::{Message, MessageParser};
use pimalaya_toolbox::terminal::printer::{Message as PrinterMessage, Printer};

use crate::{gmail::account::GmailAccount, gmail_run};

#[derive(Debug, Parser)]
pub struct GmailMessageReadCommand {
    #[arg(value_name = "ID", required = true)]
    pub ids: Vec<String>,

    #[arg(long)]
    pub html: bool,
}

impl GmailMessageReadCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;
        let mut contents = String::new();
        let total = self.ids.len();

        for (index, id) in self.ids.into_iter().enumerate() {
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
            let raw = decode_raw(raw)?;
            let Some(message) = MessageParser::new().parse(&raw) else {
                bail!(
                    "Read message `{}` error: failed to parse MIME message",
                    response.id
                );
            };

            if index > 0 {
                contents.push_str("\n\n");
            }

            if total > 1 {
                contents.push_str("# ");
                contents.push_str(&response.id);
                contents.push('\n');
            }

            let rendered = if self.html {
                format!("{}", MessageHtmlView { message })
            } else {
                format!("{}", MessagePlainView { message })
            };
            contents.push_str(rendered.trim_end());
        }

        printer.out(PrinterMessage::new(contents))
    }
}

struct MessagePlainView<'a> {
    message: Message<'a>,
}

impl fmt::Display for MessagePlainView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, part) in self.message.text_bodies().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }
            if let Some(contents) = part.text_contents() {
                write!(f, "{}", contents.trim_end())?;
            }
        }

        Ok(())
    }
}

struct MessageHtmlView<'a> {
    message: Message<'a>,
}

impl fmt::Display for MessageHtmlView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, part) in self.message.html_bodies().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }
            if let Some(contents) = part.text_contents() {
                write!(f, "{}", contents.trim_end())?;
            }
        }

        Ok(())
    }
}
