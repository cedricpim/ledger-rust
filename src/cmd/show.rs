use chrono::naive::NaiveDate;
use serde::Deserialize;

use crate::config::Config;
use crate::entity::line::Liner;
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::{repository, util, CliResult};

static USAGE: &'static str = "
Shows list of entries that match the filters.

This command will print the list of entries that match the filters provided to the output defined
(by default, STDOUT will be used). The list of entries used can either be from ledger CSV or from
networth CSV.

Usage:
    ledger show [options] [--categories=<categories>...]

Options:
    -y, --year=<year>                   Select entries that occurred on the year
    -m, --month=<month>                 Select entries that occurred on the month
    -f, --from=<from>                   Select entries that occurred after the date
    -t, --till=<till>                   Select entries that occurred before the date
    -c, --categories=<categories>       Select entries that don't match the categories
    -C, --currency=<currency>           Display entries on the currency
    -o, --output=<output>               Print selected entries to the output [default: /dev/stdout]
    -n, --networth                      Select entries from networth CSV instead of ledger CSV
    -h, --help                          Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_year: Option<i32>,
    flag_month: Option<u32>,
    flag_from: Option<NaiveDate>,
    flag_till: Option<NaiveDate>,
    flag_categories: Vec<String>,
    flag_output: String,
    flag_currency: String,
    flag_networth: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.show(&config)
}

impl Args {
    fn show(&self, config: &Config) -> CliResult<()> {
        let resource = repository::Resource::new(&config, self.flag_networth)?;

        let filter = Filter::new(
            self.flag_year,
            self.flag_month,
            self.flag_from,
            self.flag_till,
            self.flag_categories.clone(),
        );

        let currency = util::currency(&self.flag_currency)?;

        let exchange = Exchange::new(&config)?;

        let mut wtr = csv::Writer::from_path(&self.flag_output)?;

        resource.line(&mut |record| {
            if filter.apply(&record) {
                record.exchange(&currency, &exchange)?.write(&mut wtr)?;
            };

            wtr.flush()?;

            Ok(())
        })?;

        Ok(())
    }
}
