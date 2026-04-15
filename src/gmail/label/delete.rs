use anyhow::Result;
use clap::Parser;
use io_gmail::labels::delete::{GmailLabelDelete, GmailLabelDeleteResult};
use pimalaya_toolbox::terminal::printer::{Message, Printer};

use crate::{gmail::account::GmailAccount, gmail_run};

#[derive(Debug, Parser)]
pub struct GmailLabelDeleteCommand {
    #[arg(value_name = "LABEL-ID")]
    pub id: String,
}

impl GmailLabelDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;
        gmail_run!(
            gmail,
            || GmailLabelDelete::new(&gmail.http_auth, &gmail.user_id, &self.id),
            GmailLabelDeleteResult
        )?;

        printer.out(Message::new(format!(
            "Label `{}` successfully deleted",
            self.id
        )))
    }
}
