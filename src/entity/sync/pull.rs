use anyhow::anyhow;
use firefly_iii::models::transaction_read::TransactionRead;
use firefly_iii::models::transaction_split::TransactionSplit;
use firefly_iii::models::TransactionTypeProperty;

use crate::config::Config;
use crate::config::FireflyOptions;
use crate::entity::entry;
use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::resource::Resource;
use crate::service::firefly::Firefly;
use crate::Mode;

pub struct Pull {
    from: i32,
    firefly: Firefly,
    transactions: Vec<Line>,
}

impl Pull {
    pub fn new(options: &FireflyOptions) -> Self {
        Self {
            from: 0,
            firefly: Firefly::new(&options.base_path, &options.token),
            transactions: Vec::new(),
        }
    }

    pub fn perform(&mut self, config: Config) -> anyhow::Result<()> {
        self.load(&config)?;

        self.pull(&config)?;

        self.store(&config)?;

        Ok(())
    }

    fn load(&mut self, config: &Config) -> anyhow::Result<()> {
        Resource::new(config, Mode::Ledger)?.line(&mut |record| {
            self.find_highest_id(record);
            Ok(())
        })?;

        Resource::new(config, Mode::Networth)?.line(&mut |record| {
            self.find_highest_id(record);
            Ok(())
        })?;

        Ok(())
    }

    fn pull(&mut self, config: &Config) -> anyhow::Result<()> {
        for transaction in self.firefly.transactions(self.from)? {
            Transaction::new(transaction, config).process(&mut |record: Line| match record {
                Line::Transaction { .. } => {
                    self.transactions.push(record);

                    Ok(())
                }
                Line::Entry { .. } => Err(anyhow!("The line {:?} could not be pulled", record)),
            })?;
        }

        Ok(())
    }

    fn store(&mut self, config: &Config) -> anyhow::Result<()> {
        let sorter = |a: &Line, b: &Line| a.date().cmp(&b.date()).then(a.id().cmp(&b.id()));

        self.transactions.sort_by(sorter);
        Resource::new(config, Mode::Ledger)?.book(&self.transactions)?;

        Ok(())
    }

    fn find_highest_id(&mut self, record: &Line) {
        let parsed_id = record.id().parse::<i32>().unwrap_or_default();

        if self.from < parsed_id {
            self.from = parsed_id;
        };
    }
}

pub struct Transaction {
    splits: Vec<TransactionSplit>,
    transfer: String,
}

trait Pullable {
    fn lines(&self, transfer: &str) -> anyhow::Result<Vec<Line>>;
    fn build_line(
        &self,
        source: Option<String>,
        destination: Option<String>,
        amount: String,
    ) -> anyhow::Result<Line>;
}

impl Transaction {
    pub fn new(transaction: TransactionRead, config: &Config) -> Self {
        Self {
            splits: transaction.attributes.transactions,
            transfer: config.transfer.to_string(),
        }
    }

    pub fn process<F>(&mut self, action: &mut F) -> anyhow::Result<()>
    where
        F: FnMut(Line) -> anyhow::Result<()>,
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
    fn lines(&self, transfer: &str) -> anyhow::Result<Vec<Line>> {
        let result = if self._type == TransactionTypeProperty::Transfer {
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
        } else if self._type == TransactionTypeProperty::Deposit {
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
    ) -> anyhow::Result<Line> {
        let mode = if source.clone().unwrap_or_default() == entry::DEFAULT_ACCOUNT {
            Mode::Networth
        } else {
            Mode::Ledger
        };

        let attributes = match mode {
            Mode::Ledger => vec![
                source.unwrap_or_default(),
                self.date.to_string(),
                destination.unwrap_or_default(),
                self.description.to_string(),
                self.notes.clone().unwrap_or_default(),
                self.category_name.clone().unwrap_or_default(),
                amount,
                self.currency_code.clone().unwrap_or_default(),
                self.tags.clone().unwrap_or_default().join(","),
                self.transaction_journal_id.clone().unwrap_or_default(),
            ],
            Mode::Networth => vec![
                self.date.to_string(),
                Money::default().to_storage(),
                amount,
                Money::default().to_storage(),
                self.currency_code.clone().unwrap_or_default(),
                self.transaction_journal_id.clone().unwrap_or_default(),
            ],
        };

        Line::build(attributes, mode)
    }
}
