use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::gmail::{
    account::GmailAccount,
    label::{
        create::GmailLabelCreateCommand, delete::GmailLabelDeleteCommand,
        list::GmailLabelListCommand,
    },
};

#[derive(Debug, Subcommand)]
pub enum GmailLabelCommand {
    List(GmailLabelListCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(GmailLabelCreateCommand),
    #[command(visible_aliases = ["del", "remove", "rm"])]
    Delete(GmailLabelDeleteCommand),
}

impl GmailLabelCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, account),
            Self::Create(cmd) => cmd.execute(printer, account),
            Self::Delete(cmd) => cmd.execute(printer, account),
        }
    }
}
