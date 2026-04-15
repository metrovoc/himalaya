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
        // `body_text` falls back to html_to_text when only an HTML part exists,
        // so HTML-only mail is rendered as readable plain text instead of raw markup.
        let mut first = true;
        let mut pos = 0;
        while let Some(contents) = self.message.body_text(pos) {
            if !first {
                writeln!(f)?;
                writeln!(f)?;
            }
            write!(f, "{}", contents.trim_end())?;
            first = false;
            pos += 1;
        }
        Ok(())
    }
}

struct MessageHtmlView<'a> {
    message: Message<'a>,
}

impl fmt::Display for MessageHtmlView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `body_html` wraps text-only parts in minimal HTML so `--html`
        // always yields real HTML markup.
        let mut first = true;
        let mut pos = 0;
        while let Some(contents) = self.message.body_html(pos) {
            if !first {
                writeln!(f)?;
                writeln!(f)?;
            }
            write!(f, "{}", contents.trim_end())?;
            first = false;
            pos += 1;
        }
        Ok(())
    }
}
