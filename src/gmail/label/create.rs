use anyhow::Result;
use clap::Parser;
use io_gmail::labels::create::{GmailLabelCreate, GmailLabelCreateResult};
use pimalaya_toolbox::terminal::printer::{Message, Printer};

use crate::{gmail::account::GmailAccount, gmail_run};

#[derive(Debug, Parser)]
pub struct GmailLabelCreateCommand {
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl GmailLabelCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;
        let label = gmail_run!(
            gmail,
            || GmailLabelCreate::new(&gmail.http_auth, &gmail.user_id, &self.name),
            GmailLabelCreateResult
        )?;

        printer.out(Message::new(format!(
            "Label `{}` successfully created ({})",
            label.id, label.name
        )))
    }
}
