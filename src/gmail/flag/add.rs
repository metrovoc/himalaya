use anyhow::Result;
use clap::Parser;
use io_gmail::{
    labels::list::{GmailLabelsList, GmailLabelsListResult},
    messages::modify::{GmailMessageModify, GmailMessageModifyResult},
};
use pimalaya_toolbox::terminal::printer::{Message, Printer};

use crate::{
    gmail::{account::GmailAccount, flag::build_label_updates},
    gmail_run,
};

#[derive(Debug, Parser)]
pub struct GmailFlagAddCommand {
    #[arg(value_name = "ID", required = true)]
    pub ids: Vec<String>,

    #[arg(short, long, required = true, num_args = 1.., value_name = "FLAG")]
    pub flag: Vec<String>,
}

impl GmailFlagAddCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;
        let labels = gmail_run!(
            gmail,
            || GmailLabelsList::new(&gmail.http_auth, &gmail.user_id),
            GmailLabelsListResult
        )?
        .labels;
        let (add_label_ids, remove_label_ids) = build_label_updates(&self.flag, &labels, false)?;

        for id in &self.ids {
            gmail_run!(
                gmail,
                || GmailMessageModify::new(
                    &gmail.http_auth,
                    &gmail.user_id,
                    id,
                    &add_label_ids,
                    &remove_label_ids,
                ),
                GmailMessageModifyResult
            )?;
        }

        printer.out(Message::new("Flag(s) successfully added"))
    }
}
