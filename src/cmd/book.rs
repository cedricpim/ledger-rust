use serde::Deserialize;

use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;

use crate::config::Config;
use crate::line::{Line, Liner, Transaction};
use crate::{repository, util, CliResult};

static USAGE: &'static str = "
Adds a transaction to the ledger.

This command will, if used without any arguments, request all the fields that compose a single
transaction or create a transaction based in the arguments provided. It will then store the
transaction in the ledger file.

Usage:
    ledger book [options] [--transaction=<transaction>...]

Options:
    -t, --transaction=<transaction>     Define the list of values that compose a transaction, separated by comma (order: account, date, category, description, quantity, venue, amount, currency, trip).
    -h, --help                          Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_transaction: Vec<String>,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.book(config)
}

impl Args {
    fn book(&self, config: Config) -> CliResult<()> {
        let mut values = self.flag_transaction.clone();

        if self.flag_transaction.is_empty() {
            self.request_transaction_values(&mut values)?
        };

        let transaction = Transaction::build(values)?;

        self.save(config, transaction)
    }

    fn save(&self, config: Config, transaction: Transaction) -> CliResult<()> {
        let resource = repository::Resource::new(config, false)?;

        resource.apply(|file| {
            let afile = OpenOptions::new().append(true).open(file.path())?;
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(afile);
            wtr.serialize(transaction)?;
            wtr.flush()?;
            Ok(())
        })
    }

    fn request_transaction_values(&self, values: &mut Vec<String>) -> CliResult<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let line: Line = Transaction::default().into();

        for name in line.headers().iter() {
            handle
                .write_all(format!("{}: ", name).as_bytes())
                .and_then(|_v| handle.flush())?;

            let value = io::stdin()
                .lock()
                .lines()
                .next()
                .unwrap_or_else(|| Ok("".to_string()))?;

            values.push(value);
        }

        Ok(())
    }
}
