use clap::Parser;

use crate::config::Config;
use crate::entity::report::{check, general};
use crate::entity::{date::Date, total::Total};
use crate::exchange::Exchange;
use crate::filter::Filter;

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
    /// Exclude entries that match the categories
    #[arg(short, long)]
    pub exclude: Vec<String>,
    /// Display entries on the same currency (format ISO 4217)
    #[arg(short = 'C', long)]
    pub currency: Option<String>,
    /// Display report with aggregated information
    #[arg(short, long)]
    check: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.generate(&config)
}

impl Args {
    fn generate(&self, config: &Config) -> anyhow::Result<()> {
        let exchange = Exchange::new(config)?;

        let filter = Filter::report(self, config);

        if self.check {
            let report = check::Report::new(config, &filter)?;

            report.display();
        } else {
            let mut total = Total::new(self.currency.as_ref(), config, filter.end)?;

            let report = general::Report::new(self, &mut total, config, &exchange, &filter)?;

            let summary = general::Summary::new(&report, total);

            report.display();

            summary.display();
        }

        Ok(())
    }
}
