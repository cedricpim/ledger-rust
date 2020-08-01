use clap::Clap;

use std::io;
use std::io::prelude::*;

use crate::config::Config;
use crate::entity::line::Line;
use crate::resource::Resource;
use crate::CliResult;

#[derive(Clap, Debug)]
pub struct Args {
    /// Define the list of values that compose an transaction/entry
    #[clap(short, long)]
    attributes: Vec<String>,
    #[clap(
        arg_enum,
        default_value = "ledger",
        default_value_if("networth", None, "networth"),
        hidden = true
    )]
    mode: crate::Mode,
    /// Create an entry for networth CSV instead of for ledger CSV
    #[clap(short, long)]
    networth: bool,
}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.book(&config)
}

impl Args {
    fn book(&self, config: &Config) -> CliResult<()> {
        let resource = Resource::new(&config, self.mode)?;

        let mut values = self.attributes.clone();

        if values.is_empty() {
            self.collect_attributes(&mut values, &resource)?
        };

        let line = Line::build(values, self.mode)?;

        resource.book(&[line])
    }

    fn collect_attributes(&self, values: &mut Vec<String>, resource: &Resource) -> CliResult<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for name in resource.headers().iter() {
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
