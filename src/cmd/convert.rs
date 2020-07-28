use serde::Deserialize;

use std::collections::HashMap;

use crate::config::Config;
use crate::entity::line::Liner;
use crate::exchange::Exchange;
use crate::resource::Resource;
use crate::{util, CliResult};

static USAGE: &str = "
Convert other currencies to main currency of each account.

Since it isn't always possible to provide the correct currency of the money spent in each account,
this option allows the user to provide entries in any currency and then, once this is run,
converting all entries of a given account to the main currency of that account. The main currency
of each account is calculated by checking the currency of the first transaction that occurred for
each unique account.

Usage:
    ledger convert [options]

Options:
    -n, --networth  Convert entries from networth CSV instead of ledger CSV
    -h, --help      Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_networth: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.convert(&config)
}

impl Args {
    fn convert(&self, config: &Config) -> CliResult<()> {
        let resource = Resource::new(&config, self.flag_networth)?;

        let exchange = Exchange::new(&config)?;

        let mut wtr = csv::Writer::from_path(&resource.tempfile)?;

        let mut currencies: HashMap<String, String> = HashMap::new();

        resource.line(&mut |record| {
            let entry = currencies
                .entry(record.account())
                .or_insert_with(|| record.currency().code());

            record
                .exchange(util::currency(&entry, &config)?, &exchange)?
                .write(&mut wtr)?;

            wtr.flush()?;

            Ok(())
        })?;

        Ok(())
    }
}
