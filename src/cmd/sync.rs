use serde::Deserialize;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;

use crate::config::Config;
use crate::config::FireflyOptions;
use crate::entity::line::{Line, Liner};
use crate::repository::Resource;
use crate::service::firefly::Firefly;
use crate::error::CliError;
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
                transfer: Transfer { from: None, to: None },
            }
            .perform(config),
            None => Ok(crate::wout!("{}", MISSING_KEY)),
        }
    }
}

struct Sync {
    firefly: Firefly,
    options: FireflyOptions,
    currencies: HashSet<String>,
    accounts: HashMap<(String, String), String>,
    transfer: Transfer,
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

    fn records(&self) -> CliResult<(Line, Line)> {
        Ok(
            (
                self.from.clone().ok_or(CliError::MissingTransferMember)?,
                self.to.clone().ok_or(CliError::MissingTransferMember)?,
            )
        )
    }

    fn clear(&mut self) {
        self.from = None;
        self.to = None;
    }
}

impl Sync {
    fn perform(&mut self, config: Config) -> CliResult<()> {
        self.load()?;

        self.sync_transactions(&config)?;

        Ok(())
    }

    // TODO: Move ids of accounts to i32
    // TODO: clear from/to on Transfer#records
    // TODO: Move return of strings to firefly
    // TODO: Work on transfers
    fn sync_transactions(&mut self, config: &Config) -> CliResult<()> {
        let resource = Resource::new(&config, false)?;
        let temp_resource = Resource::new(&config, false)?;

        resource.apply(|file| {
            let mut wtr = csv::WriterBuilder::new().from_path(file.path())?;

            temp_resource.line(&mut |record| {
                self.process_currency(&record)?;

                if record.id().is_empty() {
                    let (id, lines) = self.process_record(&record)?;

                    for mut line in lines {
                        line.set_id(id.to_string());
                        line.write(&mut wtr)?;
                    };
                } else {
                    record.write(&mut wtr)?;
                }

                wtr.flush()?;

                Ok(())
            })?;

            Ok(())
        })?;

        Ok(())
    }

    fn process_record(&mut self, record: &Line) -> CliResult<(String, Vec<Line>)> {
        let (mut id, mut lines) = (String::new(), vec![]);

        if record.category() == self.options.transfers {
            self.transfer.add(record.clone())?;

            if self.transfer.ready() {
                let (from, to) = self.transfer.records()?;

                id = self.process_transfer(&from, &to)?;

                lines.push(from);
                lines.push(to);

                self.transfer.clear();
            }
        } else {
            id = self.process_transaction(&record)?;

            lines.push(record.clone());
        }

        Ok((id, lines))
    }

    fn process_transfer(&mut self, from: &Line, to: &Line) -> CliResult<String> {
        let from_account_id = self.process_account(&from, from.account(), false)?;
        let to_account_id = self.process_account(&to, to.account(), false)?;

        let transfer = self.firefly.create_transfer(&from, &to, from_account_id, to_account_id)?;

        if let Some(val) = transfer.data {
            Ok(val.id)
        } else {
            Ok(String::new())
        }
    }

    fn process_transaction(&mut self, record: &Line) -> CliResult<String> {
       if record.category() == self.options.opening_balance {
            self.process_account(&record, record.account(), true)
        } else {
            let balancesheet_id = self.process_account(&record, record.account(), false)?;
            let profit_loss_id = self.process_account(&record, record.category(), false)?;
            let transaction = self.firefly.create_transaction(&record, balancesheet_id, profit_loss_id)?;

            if let Some(val) = transaction.data {
                Ok(val.id)
            } else {
                Ok(String::new())
            }
        }
    }

    fn process_account(&mut self, record: &Line, account_name: String, with_balance: bool) -> CliResult<String> {
        let _type = self.firefly.type_for(&record, record.account() == account_name);
        let key = (account_name.to_string(), _type.to_string());

        match self.accounts.entry(key) {
            Entry::Occupied(v) => Ok(v.get().to_string()),
            Entry::Vacant(v) => {
                let account = self.firefly.create_account(&record, account_name, with_balance, _type)?;

                if let Some(val) = account.data {
                    v.insert(val.id.to_string());

                    if with_balance {
                        Ok(format!("B{}", val.id))
                    } else {
                        Ok(val.id)
                    }
                } else {
                    Ok(String::new())
                }
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

    fn load(&mut self) -> CliResult<()> {
        self.firefly
            .default_currency(self.options.currency.to_string())?;

        for account in self.firefly.accounts()? {
            let info = (account.attributes.name.to_string(), account.attributes._type.to_string());

            self.accounts.entry(info).or_insert_with(|| account.id);
        }

        for currency in self.firefly.currencies()? {
            if currency.attributes.enabled.unwrap_or_default() {
                self.currencies.insert(currency.attributes.code);
            }
        }

        Ok(())
    }
}
