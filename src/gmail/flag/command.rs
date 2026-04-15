use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::gmail::{
    account::GmailAccount,
    flag::{add::GmailFlagAddCommand, remove::GmailFlagRemoveCommand},
};

#[derive(Debug, Subcommand)]
pub enum GmailFlagCommand {
    Add(GmailFlagAddCommand),
    Remove(GmailFlagRemoveCommand),
}

impl GmailFlagCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        match self {
            Self::Add(cmd) => cmd.execute(printer, account),
            Self::Remove(cmd) => cmd.execute(printer, account),
        }
    }
}
