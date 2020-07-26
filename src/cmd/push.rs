use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use crate::config::Config;
use crate::config::FireflyOptions;
use crate::entity::line::{Line, Liner};
use crate::entity::push::{AccountData, Ledger, Networth, Pushable};
use crate::error::CliError;
use crate::filter::Filter;
use crate::repository::Resource;
use crate::service::firefly::Firefly;
use crate::{util, CliResult};

static USAGE: &str = "
Push entries and transactions to Firefly III.

This command will push any new entries and transactions into Firefly III (if the configuration file
is set for Firefly). In order to keep track of the already pushed transactions/entries, they will
be marked with the returned id and stored back in the CSV. For setting up the configuration with
Firefly, ensure that the key \"firefly\" has a valid access token in the configuration file.

Usage:
    ledger push [options]

Options:
    -h, --help          Display this message
";

static MISSING_KEY: &str = "There is no key set up";
static PROGRESS_BAR_FORMAT: &str = "{spinner:.green}▕{wide_bar:.cyan}▏{percent}% ({eta})";
static PROGRESS_BAR_CHARS: &str = "█▉▊▋▌▍▎▏  ";

#[derive(Debug, Deserialize)]
struct Args {}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.push(config)
}

impl Args {
    fn push(&self, config: Config) -> CliResult<()> {
        match &config.firefly {
            Some(val) => Push::new(&val, &config)?.perform(config),
            None => crate::werr!(2, "{}", MISSING_KEY),
        }
    }
}

pub struct Push {
    user: i32,
    firefly: Firefly,
    options: FireflyOptions,
    currencies: HashSet<String>,
    accounts: HashMap<(String, String), i32>,
}

impl Push {
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
                if record.pushable() {
                    pb.inc(1);
                };

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

    fn new(options: &FireflyOptions, config: &Config) -> CliResult<Self> {
        let client = Firefly::new(&options.base_path, &options.token);

        Ok(Self {
            user: client.user()?.parse::<i32>()?,
            firefly: client,
            options: FireflyOptions::build(&options, &config),
            currencies: HashSet::new(),
            accounts: HashMap::new(),
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

    fn perform(&mut self, config: Config) -> CliResult<()> {
        let pb = ProgressBar::new(config.total_pushable_lines()? as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(PROGRESS_BAR_FORMAT)
                .progress_chars(PROGRESS_BAR_CHARS),
        );

        self.load()?;

        let filter = Filter::push(&config);
        let client = Firefly::new(&self.options.base_path, &self.options.token);

        let mut ledger = Ledger::new(self.user, &filter, &client, self.options.clone());
        self.push(&config, false, &mut ledger, &pb)?;

        let mut networth = Networth::new(self.user, &filter, &client);
        self.push(&config, true, &mut networth, &pb)?;

        Ok(())
    }

    fn push<'a, T>(
        &mut self,
        config: &Config,
        networth: bool,
        entity: &'a mut T,
        pb: &ProgressBar,
    ) -> CliResult<()>
    where
        T: Pushable<'a>,
    {
        Self::process(&config, networth, &pb, &mut |record, error| match error {
            None => {
                let result = self
                    .process_currency(&record)
                    .and(entity.process(record, self));

                let handle_error = |e: CliError| -> CliResult<(String, Vec<Line>)> {
                    *error = Some(e);
                    Ok(entity.previous().map_or_else(
                        || record.pushed(),
                        |v| (record.id(), vec![v.clone(), record.clone()]),
                    ))
                };

                result.or_else(handle_error)
            }
            Some(_) => Ok(record.pushed()),
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
                format!("{:?}", account.attributes._type),
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
