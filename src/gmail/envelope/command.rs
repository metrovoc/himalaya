use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::gmail::{
    account::GmailAccount,
    envelope::{get::GmailEnvelopeGetCommand, list::GmailEnvelopeListCommand},
};

#[derive(Debug, Subcommand)]
pub enum GmailEnvelopeCommand {
    List(GmailEnvelopeListCommand),
    Get(GmailEnvelopeGetCommand),
}

impl GmailEnvelopeCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, account),
            Self::Get(cmd) => cmd.execute(printer, account),
        }
    }
}
