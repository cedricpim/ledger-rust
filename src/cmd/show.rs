use clap::Clap;

use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::line::Liner;
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::resource::Resource;
use crate::{util, CliResult};

#[derive(Clap, Debug)]
pub struct Args {
    /// Select entries that occurred on the year
    #[clap(short, long)]
    pub year: Option<i32>,
    /// Select entries that occurred on the month
    #[clap(short, long)]
    pub month: Option<u32>,
    /// Select entries that occurred after the date
    #[clap(short, long)]
    pub from: Option<Date>,
    /// Select entries that occurred before the date
    #[clap(short, long)]
    pub till: Option<Date>,
    /// Select entries that match the categories
    #[clap(short, long)]
    pub categories: Vec<String>,
    /// Display entries on the same currency (format ISO 4217)
    #[clap(short = 'C', long)]
    currency: Option<String>,
    /// Print selected entries to the output
    #[clap(short, long, default_value = "/dev/stdout")]
    output: String,
    #[clap(
        arg_enum,
        default_value = "ledger",
        default_value_if("networth", None, "networth"),
        hidden = true
    )]
    mode: crate::Mode,
    /// Select entries from networth CSV instead of ledger CSV
    #[clap(short, long)]
    networth: bool,
}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.show(&config)
}

impl Args {
    fn show(&self, config: &Config) -> CliResult<()> {
        let mut resource = Resource::new(&config, self.mode)?;

        let filter = Filter::show(&self);

        let currency = util::currency(self.currency.as_ref(), &config)?;

        let exchange = Exchange::new(&config)?;

        let mut wtr = csv::Writer::from_path(&self.output)?;

        resource.line(&mut |record| {
            if filter.display(&record) {
                record.exchange(currency, &exchange)?.write(&mut wtr)?;
            };

            wtr.flush()?;

            Ok(())
        })?;

        Ok(())
    }
}
