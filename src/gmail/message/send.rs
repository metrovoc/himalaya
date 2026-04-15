use std::{
    fmt,
    io::{stdin, BufRead, IsTerminal},
};

use anyhow::Result;
use clap::Parser;
use io_gmail::messages::send::{GmailMessageSend, GmailMessageSendResult};
use log::warn;
use pimalaya_toolbox::terminal::printer::Printer;
use serde::Serialize;

use crate::{gmail::account::GmailAccount, gmail_run};

#[derive(Debug, Parser)]
pub struct GmailMessageSendCommand {
    #[arg(trailing_var_arg = true)]
    #[arg(name = "message", value_name = "MESSAGE")]
    pub message: Vec<String>,
}

impl GmailMessageSendCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;

        let raw: Vec<u8> = if stdin().is_terminal() || printer.is_json() {
            self.message
                .join(" ")
                .replace('\r', "")
                .replace('\n', "\r\n")
                .into_bytes()
        } else {
            stdin()
                .lock()
                .lines()
                .map_while(Result::ok)
                .collect::<Vec<String>>()
                .join("\r\n")
                .into_bytes()
        };

        if raw.len() > 5 * 1024 * 1024 {
            warn!("message exceeds the 5 MiB simple Gmail send limit; the API may reject it");
        }

        let response = gmail_run!(
            gmail,
            || GmailMessageSend::new(&gmail.http_auth, &gmail.user_id, &raw),
            GmailMessageSendResult
        )?;

        printer.out(SentMessage { id: response.id })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SentMessage {
    pub id: String,
}

impl fmt::Display for SentMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sent {}", self.id)
    }
}
