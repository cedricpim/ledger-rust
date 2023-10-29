use anyhow::anyhow;
use clap::Parser;

use std::path::Path;

use crate::config::Config;

use crate::resource::Resource;

static SUCCESS: &str = "Generated default file on";

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(
        value_enum,
        default_value = "ledger",
        default_value_if("networth", "", Some("networth")),
        hide = true
    )]
    mode: crate::Mode,
    /// Create networth CSV instead of ledger CSV
    #[arg(short, long)]
    networth: bool,
    /// Create the initial file, overriding existing one
    #[arg(short, long)]
    force: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.create(&config)
}

impl Args {
    fn create(&self, config: &Config) -> anyhow::Result<()> {
        let resource = Resource::new(config, self.mode)?;

        if Path::new(&resource.filepath).exists() && !self.force {
            Err(anyhow!(
                "File {} already exists, use --force to overwrite it",
                resource.filepath
            ))
        } else {
            resource.create()?;
            crate::wout!("{} {}", SUCCESS, resource.filepath);
            Ok(())
        }
    }
}
