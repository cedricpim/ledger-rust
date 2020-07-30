use firefly_iii::models::transaction_read::TransactionRead;
use firefly_iii::models::transaction_split::TransactionSplit;
use firefly_iii::models::transaction_split::Type;

use crate::config::Config;
use crate::entity::entry;
use crate::entity::line::Line;
use crate::entity::money::Money;
use crate::CliResult;

pub struct Transaction {
    splits: Vec<TransactionSplit>,
    transfer: String,
}

trait Pullable {
    fn lines(&self, transfer: &str) -> CliResult<Vec<Line>>;
    fn build_line(
        &self,
        source: Option<String>,
        destination: Option<String>,
        amount: String,
    ) -> CliResult<Line>;
}

impl Transaction {
    pub fn new(transaction: TransactionRead, config: &Config) -> Self {
        Self {
            splits: transaction.attributes.transactions,
            transfer: config.transfer.to_string(),
        }
    }

    pub fn process<F>(&mut self, action: &mut F) -> CliResult<()>
    where
        F: FnMut(Line) -> CliResult<()>,
    {
        for split in &self.splits {
            for line in split.lines(&self.transfer)? {
                action(line)?;
            }
        }

        Ok(())
    }
}

impl Pullable for TransactionSplit {
    fn lines(&self, transfer: &str) -> CliResult<Vec<Line>> {
        let result = if self._type == Some(Type::Transfer) {
            vec![
                self.build_line(
                    self.source_name.clone(),
                    Some(transfer.to_string()),
                    format!("-{}", self.amount),
                )?,
                self.build_line(
                    self.destination_name.clone(),
                    Some(transfer.to_string()),
                    format!("+{}", self.amount),
                )?,
            ]
        } else if self._type == Some(Type::Deposit) {
            vec![self.build_line(
                self.destination_name.clone(),
                self.source_name.clone(),
                format!("+{}", self.amount),
            )?]
        } else {
            vec![self.build_line(
                self.source_name.clone(),
                self.destination_name.clone(),
                format!("-{}", self.amount),
            )?]
        };

        Ok(result)
    }

    fn build_line(
        &self,
        source: Option<String>,
        destination: Option<String>,
        amount: String,
    ) -> CliResult<Line> {
        let networth = source.clone().unwrap_or_default() == entry::DEFAULT_ACCOUNT;

        let attributes = if networth {
            vec![
                self.date.to_string(),
                Money::default().to_storage(),
                amount,
                Money::default().to_storage(),
                self.currency_code.clone().unwrap_or_default(),
                self.transaction_journal_id.unwrap_or_default().to_string(),
            ]
        } else {
            vec![
                source.unwrap_or_default(),
                self.date.to_string(),
                destination.unwrap_or_default(),
                self.description.to_string(),
                self.notes.clone().unwrap_or_default(),
                self.category_name.clone().unwrap_or_default(),
                amount,
                self.currency_code.clone().unwrap_or_default(),
                self.tags.clone().unwrap_or_else(|| vec![]).join(","),
                self.transaction_journal_id.unwrap_or_default().to_string(),
            ]
        };

        Line::build(attributes, networth)
    }
}
