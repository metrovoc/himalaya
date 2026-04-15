use anyhow::Result;
use clap::Parser;
use io_gmail::messages::trash::{GmailMessageTrash, GmailMessageTrashResult};
use pimalaya_toolbox::terminal::printer::{Message, Printer};

use crate::{gmail::account::GmailAccount, gmail_run};

#[derive(Debug, Parser)]
pub struct GmailMessageTrashCommand {
    #[arg(value_name = "ID", required = true)]
    pub ids: Vec<String>,
}

impl GmailMessageTrashCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;

        for id in &self.ids {
            gmail_run!(
                gmail,
                || GmailMessageTrash::new(&gmail.http_auth, &gmail.user_id, id),
                GmailMessageTrashResult
            )?;
        }

        printer.out(Message::new("Message(s) successfully moved to trash"))
    }
}
