use serde::Deserialize;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use crate::config::Config;
use crate::config::FireflyOptions;
use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::entity::sync::Account;
use crate::error::CliError;
use crate::repository::Resource;
use crate::service::firefly::Firefly;
use crate::{util, CliResult};

static USAGE: &str = "
Sync entries and transactions with Firefly III.

This command will sync any new entries and transactions into Firefly III (if the configuration file
is set for Firefly). In order to keep track of the already synced transactions/entries, they will
be marked with the returned id and stored back in the CSV. For setting up the configuration with
Firefly, ensure that the key \"firefly\" has a valid access token in the configuration file.

Usage:
    ledger sync [options]

Options:
    -h, --help          Display this message
";

static MISSING_KEY: &str = "There is no synchronization set up";

#[derive(Debug, Deserialize)]
struct Args {}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.sync(config)
}

impl Args {
    fn sync(&self, config: Config) -> CliResult<()> {
        match config.firefly.clone() {
            Some(val) => Sync {
                firefly: Firefly::new(val.token.to_string()),
                options: val,
                currencies: HashSet::new(),
                accounts: HashMap::new(),
                investments: None,
                transfer: Transfer {
                    from: None,
                    to: None,
                },
            }
            .perform(config),
            None => crate::werr!(2, "{}", MISSING_KEY),
        }
    }
}

struct Transfer {
    from: Option<Line>,
    to: Option<Line>,
}

impl Transfer {
    fn add(&mut self, record: Line) -> CliResult<()> {
        if self.from.is_none() {
            self.from = Some(record);
        } else if self.to.is_none() {
            self.to = Some(record);
        } else {
            return Err(CliError::IncorrectTransfer);
        };

        Ok(())
    }

    fn ready(&self) -> bool {
        self.from.is_some() && self.to.is_some()
    }

    fn records(&mut self) -> CliResult<(Line, Line)> {
        Ok((
            std::mem::replace(&mut self.from, None).ok_or(CliError::MissingTransferMember)?,
            std::mem::replace(&mut self.to, None).ok_or(CliError::MissingTransferMember)?,
        ))
    }
}

struct Sync {
    firefly: Firefly,
    options: FireflyOptions,
    currencies: HashSet<String>,
    accounts: HashMap<(String, String), i32>,
    investments: Option<Money>,
    transfer: Transfer,
}

impl Sync {
    fn perform(&mut self, config: Config) -> CliResult<()> {
        self.load()?;

        self.sync_transactions(&config)?;
        self.sync_investments(&config)?;

        Ok(())
    }

    fn sync_investments(&mut self, config: &Config) -> CliResult<()> {
        let resource = Resource::new(&config, true)?;
        let temp_resource = Resource::new(&config, true)?;

        resource.apply(|file| {
            let mut wtr = csv::WriterBuilder::new().from_path(file.path())?;

            temp_resource.line(&mut |record| {
                self.process_investment(record)?;

                record.write(&mut wtr)?;
                wtr.flush()?;

                Ok(())
            })
        })
    }

    fn process_investment(&mut self, record: &mut Line) -> CliResult<()> {
        self.process_currency(&record)?;

        if !record.investment().zero() {
            if record.id().is_empty() {
                let id = self.process_transaction(
                    &record,
                    record.investment() - self.investments.unwrap_or_default(),
                )?;

                record.set_id(id);
            }

            self.investments = Some(record.investment());
        }

        Ok(())
    }

    fn sync_transactions(&mut self, config: &Config) -> CliResult<()> {
        let resource = Resource::new(&config, false)?;
        let temp_resource = Resource::new(&config, false)?;

        resource.apply(|file| {
            let mut wtr = csv::WriterBuilder::new().from_path(file.path())?;

            temp_resource.line(&mut |record| {
                let (id, lines) = self.process_record(record)?;

                for mut line in lines {
                    line.set_id(id.to_string());
                    line.write(&mut wtr)?;
                    wtr.flush()?;
                }

                Ok(())
            })
        })
    }

    fn process_record(&mut self, record: &Line) -> CliResult<(String, Vec<Line>)> {
        self.process_currency(&record)?;

        let (mut id, mut lines) = (String::new(), vec![]);

        if record.id().is_empty() && !record.date().future() {
            if record.category() == self.options.transfers {
                self.transfer.add(record.clone())?;

                if self.transfer.ready() {
                    let (from, to) = self.transfer.records()?;

                    id = self.process_transfer(&from, &to)?;

                    lines.push(from);
                    lines.push(to);
                }
            } else {
                id = self.process_transaction(&record, record.amount())?;

                lines.push(record.clone());
            }
        } else {
            lines.push(record.clone());
        }

        Ok((id, lines))
    }

    fn process_transfer(&mut self, from: &Line, to: &Line) -> CliResult<String> {
        let from_id = self.process_account(Account::new(&from, from.account(), None))?;
        let to_id = self.process_account(Account::new(&to, to.account(), None))?;

        self.firefly
            .create_transaction(&from, Some(&to), from_id, to_id, from.amount(), true)
            .map_err(CliError::from)
    }

    fn process_transaction(&mut self, record: &Line, value: Money) -> CliResult<String> {
        if self.new_account_with_balance(&record) {
            self.process_account(Account::new(&record, record.account(), Some(value)))
                .map(|v| v.to_string())
        } else {
            let (one_side, other_side) = Account::doubleside(&record, Some(value));

            let balancesheet_id = self.process_account(one_side)?;
            let profit_loss_id = self.process_account(other_side)?;

            self.firefly
                .create_transaction(&record, None, balancesheet_id, profit_loss_id, value, false)
                .map_err(CliError::from)
        }
    }

    fn process_account(&mut self, account: Account) -> CliResult<i32> {
        match self.accounts.entry(account.key()) {
            Entry::Occupied(v) => Ok(*v.get()),
            Entry::Vacant(v) => {
                let id = self.firefly.create_account(account)?;

                let parsed_id = id.parse::<i32>().unwrap_or_default();

                v.insert(parsed_id);

                Ok(parsed_id)
            }
        }
    }

    fn process_currency(&mut self, record: &Line) -> CliResult<()> {
        if !self.currencies.contains(&record.currency().code()) {
            self.firefly.enable_currency(record.currency().code())?;
            self.currencies.insert(record.currency().code());
        }

        Ok(())
    }

    fn new_account_with_balance(&self, record: &Line) -> bool {
        (record.transaction() && record.category() == self.options.opening_balance)
            || (record.entry() && self.investments.is_none())
    }

    fn load(&mut self) -> CliResult<()> {
        self.firefly
            .default_currency(self.options.currency.to_string())?;

        for account in self.firefly.accounts()? {
            let info = (
                account.attributes.name.to_string(),
                account.attributes._type.to_string(),
            );

            let id = account.id.parse::<i32>().map_err(CliError::from)?;

            self.accounts.entry(info).or_insert_with(|| id);
        }

        for currency in self.firefly.currencies()? {
            if currency.attributes.enabled.unwrap_or_default() {
                self.currencies.insert(currency.attributes.code);
            }
        }

        Ok(())
    }
}
