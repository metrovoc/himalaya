use anyhow::Result;
use clap::Parser;
use io_gmail::messages::delete::{GmailMessageDelete, GmailMessageDeleteResult};
use pimalaya_toolbox::terminal::printer::{Message, Printer};

use crate::{gmail::account::GmailAccount, gmail_run};

#[derive(Debug, Parser)]
pub struct GmailMessageDeleteCommand {
    #[arg(value_name = "ID", required = true)]
    pub ids: Vec<String>,
}

impl GmailMessageDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;

        for id in &self.ids {
            gmail_run!(
                gmail,
                || GmailMessageDelete::new(&gmail.http_auth, &gmail.user_id, id),
                GmailMessageDeleteResult
            )?;
        }

        printer.out(Message::new("Message(s) permanently deleted"))
    }
}
