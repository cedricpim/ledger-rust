use clap::Parser;

use std::collections::HashMap;

use crate::config::Config;
use crate::entity::line::Liner;
use crate::exchange::Exchange;
use crate::resource::Resource;
use crate::util;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(
        value_enum,
        default_value = "ledger",
        default_value_if("networth", "true", Some("networth")),
        hide = true
    )]
    mode: crate::Mode,
    /// Convert entries from networth CSV instead of ledger CSV
    #[arg(short, long)]
    networth: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.convert(&config)
}

impl Args {
    fn convert(&self, config: &Config) -> anyhow::Result<()> {
        let mut resource = Resource::new(config, self.mode)?;

        let exchange = Exchange::new(config)?;

        let mut wtr = csv::Writer::from_path(&resource.tempfile)?;

        let mut currencies: HashMap<String, String> = HashMap::new();

        resource.line(&mut |record| {
            let entry = currencies
                .entry(record.account())
                .or_insert_with(|| record.currency().code());

            record
                .exchange(util::currency(Some(entry), config)?, &exchange)?
                .write(&mut wtr)?;

            wtr.flush()?;

            Ok(())
        })?;

        Ok(())
    }
}
