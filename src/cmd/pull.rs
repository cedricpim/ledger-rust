use clap::Clap;

use crate::config::Config;
use crate::config::FireflyOptions;
use crate::entity::line::{Line, Liner};
use crate::entity::pull::Transaction;
use crate::error::CliError;
use crate::resource::Resource;
use crate::service::firefly::Firefly;
use crate::{CliResult, Mode};

static MISSING_KEY: &str = "There is no key set up";

#[derive(Clap, Debug, Default)]
pub struct Args {}

pub fn run(args: Args) -> CliResult<()> {
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
        Resource::new(&config, Mode::Ledger)?.line(&mut |record| self.find_highest_id(record))?;

        Resource::new(&config, Mode::Networth)?.line(&mut |record| self.find_highest_id(record))?;

        Ok(())
    }

    fn pull(&mut self, config: &Config) -> CliResult<()> {
        for transaction in self.firefly.transactions(self.from)? {
            Transaction::new(transaction, &config).process(&mut |record: Line| match record {
                Line::Transaction { .. } => {
                    self.transactions.push(record);

                    Ok(())
                }
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
        Resource::new(&config, Mode::Ledger)?.book(&self.transactions)?;

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
