use clap::Parser;

use crate::config::Config;
use crate::entity::report::networth;
use crate::exchange::Exchange;
use crate::util;

#[derive(Parser, Debug)]
pub struct Args {
    /// Display entries on the same currency (format ISO 4217)
    #[arg(short, long)]
    currency: Option<String>,
    /// Save the total networth to the networth CSV
    #[arg(short, long)]
    save: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.generate(config)
}

impl Args {
    fn generate(&self, config: Config) -> anyhow::Result<()> {
        let exchange = Exchange::new(&config)?;

        let currency = util::currency(self.currency.as_ref(), &config)?;

        let report = networth::Report::new(config, exchange, currency)?;

        if self.save {
            report.save()?
        } else {
            report.display()
        };

        Ok(())
    }
}
