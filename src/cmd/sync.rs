use serde::Deserialize;

use crate::config::Config;
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
pub struct Args {}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.sync(config)
}

impl Args {
    pub fn sync(&self, config: Config) -> CliResult<()> {
        if config.firefly.is_none() {
            crate::wout!("{}", MISSING_KEY);
            return Ok(())
        };

        Ok(())
    }
}
