use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::gmail::{
    account::GmailAccount, envelope::command::GmailEnvelopeCommand,
    flag::command::GmailFlagCommand, label::command::GmailLabelCommand,
    message::command::GmailMessageCommand,
};

#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GmailCommand {
    #[command(subcommand)]
    #[command(alias = "labels")]
    Label(GmailLabelCommand),
    #[command(subcommand)]
    #[command(alias = "envelopes")]
    Envelope(GmailEnvelopeCommand),
    #[command(subcommand)]
    #[command(aliases = ["messages", "msgs", "msg"])]
    Message(GmailMessageCommand),
    #[command(subcommand)]
    #[command(alias = "flags")]
    Flag(GmailFlagCommand),
}

impl GmailCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        match self {
            Self::Label(cmd) => cmd.execute(printer, account),
            Self::Envelope(cmd) => cmd.execute(printer, account),
            Self::Message(cmd) => cmd.execute(printer, account),
            Self::Flag(cmd) => cmd.execute(printer, account),
        }
    }
}
