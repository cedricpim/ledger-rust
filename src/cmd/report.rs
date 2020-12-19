use clap::Clap;

use crate::config::Config;
use crate::entity::{date::Date, total::Total};
use crate::entity::reporting::{general, movement};
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::CliResult;

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
    /// Exclude entries that match the categories
    #[clap(short, long)]
    pub exclude: Vec<String>,
    /// Display entries on the same currency (format ISO 4217)
    #[clap(short, long)]
    pub currency: Option<String>,
    /// Display report with information aggregated by movement
    #[clap(short, long)]
    movements: bool,
}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.generate(&config)
}

impl Args {
    fn generate(&self, config: &Config) -> CliResult<()> {
        let exchange = Exchange::new(&config)?;

        let filter = Filter::report(&self, &config);

        if self.movements {
            let report = movement::Report::new(&config, &filter)?;

            report.display();
        } else {
            let mut total = Total::new(self.currency.as_ref(), &config, filter.end)?;

            let report = general::Report::new(&self, &mut total, &config, &exchange, &filter)?;

            let summary = general::Summary::new(&report, total);

            report.display();

            summary.display();
        }

        Ok(())
    }
}
