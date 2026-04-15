use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, ContentArrangement, Row, Table};
use io_gmail::{
    labels::list::{GmailLabelsList, GmailLabelsListResult},
    types::label::Label,
};
use pimalaya_toolbox::terminal::printer::Printer;
use serde::Serialize;

use crate::{gmail::account::GmailAccount, gmail_run};

#[derive(Debug, Parser)]
pub struct GmailLabelListCommand;

impl GmailLabelListCommand {
    pub fn execute(self, printer: &mut impl Printer, account: GmailAccount) -> Result<()> {
        let mut gmail = account.new_gmail_session()?;
        let response = gmail_run!(
            gmail,
            || GmailLabelsList::new(&gmail.http_auth, &gmail.user_id),
            GmailLabelsListResult
        )?;

        printer.out(LabelsTable {
            preset: account.table_preset,
            arrangement: account.table_arrangement,
            labels: response.labels,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LabelsTable {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    pub labels: Vec<Label>,
}

impl fmt::Display for LabelsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("TYPE"),
                Cell::new("UNREAD"),
                Cell::new("TOTAL"),
            ]));

        for label in &self.labels {
            let mut row = Row::new();
            row.max_height(1);
            row.add_cell(Cell::new(&label.id));
            row.add_cell(Cell::new(&label.name));
            row.add_cell(Cell::new(label.label_type.as_deref().unwrap_or("")));
            row.add_cell(Cell::new(
                label
                    .messages_unread
                    .map(|count| count.to_string())
                    .unwrap_or_default(),
            ));
            row.add_cell(Cell::new(
                label
                    .messages_total
                    .map(|count| count.to_string())
                    .unwrap_or_default(),
            ));
            table.add_row(row);
        }

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
