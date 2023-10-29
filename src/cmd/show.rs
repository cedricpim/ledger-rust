use clap::Parser;

use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::line::Liner;
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::resource::Resource;
use crate::util;

#[derive(Parser, Debug)]
pub struct Args {
    /// Select entries that occurred on the year
    #[arg(short, long)]
    pub year: Option<i32>,
    /// Select entries that occurred on the month
    #[arg(short, long)]
    pub month: Option<u32>,
    /// Select entries that occurred after the date
    #[arg(short, long)]
    pub from: Option<Date>,
    /// Select entries that occurred before the date
    #[arg(short, long)]
    pub till: Option<Date>,
    /// Select entries that match the categories
    #[arg(short, long)]
    pub categories: Vec<String>,
    /// Display entries on the same currency (format ISO 4217)
    #[arg(short = 'C', long)]
    currency: Option<String>,
    /// Print selected entries to the output
    #[arg(short, long, default_value = "/dev/stdout")]
    output: String,
    #[arg(
        value_enum,
        default_value = "ledger",
        default_value_if("networth", "", Some("networth")),
        hide = true
    )]
    mode: crate::Mode,
    /// Select entries from networth CSV instead of ledger CSV
    #[arg(short, long)]
    networth: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.show(&config)
}

impl Args {
    fn show(&self, config: &Config) -> anyhow::Result<()> {
        let mut resource = Resource::new(config, self.mode)?;

        let filter = Filter::show(self);

        let currency = util::currency(self.currency.as_ref(), config)?;

        let exchange = Exchange::new(config)?;

        let mut wtr = csv::Writer::from_path(&self.output)?;

        resource.line(&mut |record| {
            if filter.display(record) {
                record.exchange(currency, &exchange)?.write(&mut wtr)?;
            };

            wtr.flush()?;

            Ok(())
        })?;

        Ok(())
    }
}
