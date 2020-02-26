use indicatif::ProgressBar;
use serde::Deserialize;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use crate::config::Config;
use crate::config::FireflyOptions;
use crate::entity::line::{Line, Liner};
use crate::entity::sync::{AccountData, Ledger, Networth, Syncable};
use crate::error::CliError;
use crate::filter::Filter;
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
            Some(val) => Sync::new(val)?.perform(config),
            None => crate::werr!(2, "{}", MISSING_KEY),
        }
    }
}

pub struct Sync {
    user: i32,
    firefly: Firefly,
    options: FireflyOptions,
    currencies: HashSet<String>,
    accounts: HashMap<(String, String), i32>,
}

impl Sync {
    fn process<F>(
        config: &Config,
        networth: bool,
        pb: &ProgressBar,
        action: &mut F,
    ) -> CliResult<()>
    where
        F: FnMut(&mut Line, &mut Option<CliError>) -> CliResult<(String, Vec<Line>)>,
    {
        let resource = Resource::new(&config, networth)?;
        let temp_resource = Resource::new(&config, networth)?;

        let mut error: Option<CliError> = None;

        resource.apply(|file| {
            let mut wtr = csv::WriterBuilder::new().from_path(file.path())?;

            temp_resource.line(&mut |record| {
                pb.inc(record.bytes());

                let (id, lines) = action(record, &mut error)?;

                for mut line in lines {
                    line.set_id(id.to_string());
                    line.write(&mut wtr)?;
                    wtr.flush()?;
                }

                Ok(())
            })
        })?;

        error.map_or(Ok(()), Err)
    }

    fn new(options: FireflyOptions) -> CliResult<Self> {
        let client = Firefly::new(&options.token.to_string());

        Ok(Self {
            user: client.user()?.parse::<i32>()?,
            firefly: client,
            currencies: HashSet::new(),
            accounts: HashMap::new(),
            options,
        })
    }

    pub fn account(&mut self, account: &AccountData) -> CliResult<i32> {
        match self.accounts.entry(account.key()) {
            Entry::Occupied(v) => Ok(*v.get()),
            Entry::Vacant(v) => {
                let id = self.firefly.create_account(&account)?;

                let parsed_id = id.parse::<i32>()?;

                v.insert(parsed_id);

                Ok(parsed_id)
            }
        }
    }

    // Increase the total number of bytes by 50% since we measure the number of bytes of the Struct
    // and not the number of bytes read from the file.
    fn perform(&mut self, config: Config) -> CliResult<()> {
        let pb = ProgressBar::new((config.bytes() as f64 * 1.5) as u64);

        self.load()?;

        let filter = Filter::networth(&config);
        let client = Firefly::new(&self.options.token);

        let mut ledger = Ledger::new(self.user, &filter, &client, self.options.clone());
        self.sync(&config, false, &mut ledger, &pb)?;

        let mut networth = Networth::new(self.user, &filter, &client);
        self.sync(&config, true, &mut networth, &pb)?;

        Ok(())
    }

    fn sync<'a, T>(
        &mut self,
        config: &Config,
        networth: bool,
        entity: &'a mut T,
        pb: &ProgressBar,
    ) -> CliResult<()>
    where
        T: Syncable<'a>,
    {
        Self::process(&config, networth, &pb, &mut |record, error| match error {
            None => {
                let result = self
                    .process_currency(&record)
                    .and(entity.process(record, self));

                let handle_error = |e: CliError| -> CliResult<(String, Vec<Line>)> {
                    *error = Some(e);
                    Ok(entity.previous().map_or_else(
                        || record.synced(),
                        |v| (record.id(), vec![v.clone(), record.clone()]),
                    ))
                };

                result.or_else(handle_error)
            }
            Some(_) => Ok(record.synced()),
        })
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
            let info = (
                account.attributes.name.to_string(),
                account.attributes._type.to_string(),
            );

            let id = account.id.parse::<i32>()?;

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
