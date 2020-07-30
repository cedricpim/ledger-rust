use serde::Deserialize;

use crate::config::Config;
use crate::config::FireflyOptions;
use crate::entity::line::{Line, Liner};
use crate::entity::pull::Transaction;
use crate::error::CliError;
use crate::resource::Resource;
use crate::service::firefly::Firefly;
use crate::{util, CliResult};

static USAGE: &str = "
Pull entries and transactions from Firefly III.

This command will get the latest transaction locally (by id), and pull and the entries above that
id from Firefly III (if the configuration file is set for Firefly). For setting up the
configuration with Firefly, ensure that the key \"firefly\" has a valid access token in the
configuration file.

Usage:
    ledger pull [options]

Options:
    -h, --help          Display this message
";

static MISSING_KEY: &str = "There is no key set up";

#[derive(Debug, Deserialize)]
struct Args {}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.pull(config)
}

impl Args {
    fn pull(&self, config: Config) -> CliResult<()> {
        match &config.firefly {
            Some(val) => Pull::new(&val).perform(config),
            None => crate::werr!(2, "{}", MISSING_KEY),
        }
    }
}

pub struct Pull {
    from: i32,
    firefly: Firefly,
    transactions: Vec<Line>,
}

impl Pull {
    fn new(options: &FireflyOptions) -> Self {
        Self {
            from: 0,
            firefly: Firefly::new(&options.base_path, &options.token),
            transactions: Vec::new(),
        }
    }

    fn perform(&mut self, config: Config) -> CliResult<()> {
        self.load(&config)?;

        self.pull(&config)?;

        self.store(&config)?;

        Ok(())
    }

    fn load(&mut self, config: &Config) -> CliResult<()> {
        Resource::new(&config, false)?.line(&mut |record| self.find_highest_id(record))?;

        Resource::new(&config, true)?.line(&mut |record| self.find_highest_id(record))?;

        Ok(())
    }

    fn pull(&mut self, config: &Config) -> CliResult<()> {
        for transaction in self.firefly.transactions(self.from)? {
            Transaction::new(transaction, &config).process(&mut |record: Line| match record {
                Line::Transaction { .. } => {
                    self.transactions.push(record);

                    Ok(())
                },
                Line::Entry { .. } => Err(CliError::NotPullableLine {
                    line: format!("{:?}", record),
                }),
            })?;
        }

        Ok(())
    }

    fn store(&mut self, config: &Config) -> CliResult<()> {
        let sorter = |a: &Line, b: &Line| a.date().cmp(&b.date()).then(a.id().cmp(&b.id()));

        self.transactions.sort_by(sorter);
        Resource::new(&config, false)?.book(&self.transactions)?;

        Ok(())
    }

    fn find_highest_id(&mut self, record: &Line) -> CliResult<()> {
        let parsed_id = record.id().parse::<i32>().unwrap_or_default();

        if self.from < parsed_id {
            self.from = parsed_id;
        }

        Ok(())
    }
}
