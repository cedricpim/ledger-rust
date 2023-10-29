use clap::Parser;

use std::io;
use std::io::prelude::*;

use crate::config::Config;
use crate::entity::line::Line;
use crate::resource::Resource;

// This usage is necessary because, unfortunately, clap does not handle empty values as expected
// for options that take multiple values.
// https://github.com/clap-rs/clap/issues/1740
pub static DEFAULT_EMPTY: &str = " ";

#[derive(Parser, Debug)]
pub struct Args {
    /// Define the list of values that compose an transaction/entry
    #[clap(short, long, allow_hyphen_values = true)]
    attributes: Vec<String>,
    #[clap(
        arg_enum,
        default_value = "ledger",
        default_value_if("networth", None, Some("networth")),
        hide = true
    )]
    mode: crate::Mode,
    /// Create an entry for networth CSV instead of for ledger CSV
    #[clap(short, long)]
    networth: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.book(&config)
}

impl Args {
    fn book(&self, config: &Config) -> anyhow::Result<()> {
        let mut resource = Resource::new(config, self.mode)?;

        let mut values = self.attributes.clone();

        if values.is_empty() {
            self.collect_attributes(&mut values, &resource)?
        } else {
            for val in values.iter_mut() {
                if val == DEFAULT_EMPTY {
                    val.clear()
                }
            }
        };

        let line = Line::build(values, self.mode)?;

        resource.book(&[line])
    }

    fn collect_attributes(
        &self,
        values: &mut Vec<String>,
        resource: &Resource,
    ) -> anyhow::Result<()> {
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
