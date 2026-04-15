use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::gmail::{
    account::GmailAccount,
    message::{
        delete::GmailMessageDeleteCommand, get::GmailMessageGetCommand,
        read::GmailMessageReadCommand, send::GmailMessageSendCommand,
        trash::GmailMessageTrashCommand, untrash::GmailMessageUntrashCommand,
    },
};

#[derive(Debug, Subcommand)]
pub enum GmailMessageCommand {
    Get(GmailMessageGetCommand),
    Read(GmailMessageReadCommand),
    Send(GmailMessageSendCommand),
    Trash(GmailMessageTrashCommand),
    Untrash(GmailMessageUntrashCommand),
    Delete(GmailMessageDeleteCommand),
}

impl GmailMessageCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, account),
            Self::Read(cmd) => cmd.execute(printer, account),
            Self::Send(cmd) => cmd.execute(printer, account),
            Self::Trash(cmd) => cmd.execute(printer, account),
            Self::Untrash(cmd) => cmd.execute(printer, account),
            Self::Delete(cmd) => cmd.execute(printer, account),
        }
    }
}
