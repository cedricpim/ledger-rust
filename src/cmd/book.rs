use serde::Deserialize;

use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;

use crate::config::Config;
use crate::entity::line::{Line, Liner};
use crate::repository::Resource;
use crate::{util, CliResult};

static USAGE: &str = "
Adds a transaction to the ledger.

This command will, if used without any arguments, request all the fields that compose a single
transaction/entry or create a transaction/entry based in the arguments provided. It will then store
the transaction in the ledger file (or the entry in the networth file).

Order of attribute:
    - Transaction: account, date, category, description, quantity, venue, amount, currency, trip
    - Entry: date, invested, investment, amount, currency

Usage:
    ledger book [options] [--attributes=<attributes>...]

Options:
    -a, --attributes=<attributes>     Define the list of values that compose an transaction/entry
    -n, --networth                    Create an entry for networth CSV instead of for ledger CSV
    -h, --help                        Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_attributes: Vec<String>,
    flag_networth: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.book(&config)
}

impl Args {
    fn book(&self, config: &Config) -> CliResult<()> {
        let resource = Resource::new(&config, self.flag_networth)?;

        let mut values = self.flag_attributes.clone();

        if values.is_empty() {
            self.collect_attributes(&mut values, &resource)?
        };

        let line = Line::build(values, self.flag_networth)?;

        self.save(line, resource)
    }

    fn save(&self, line: Line, resource: Resource) -> CliResult<()> {
        resource.apply(|file| {
            let afile = OpenOptions::new().append(true).open(file.path())?;
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(afile);

            line.write(&mut wtr)?;

            wtr.flush()?;
            Ok(())
        })
    }

    fn collect_attributes(&self, values: &mut Vec<String>, resource: &Resource) -> CliResult<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for name in resource.kind.headers().iter() {
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
