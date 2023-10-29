use anyhow::anyhow;
use clap::Parser;

use std::path::Path;

use crate::config::Config;

static SUCCESS: &str = "Generated default configuration file on";

#[derive(Parser, Debug)]
pub struct Args {
    /// Copy the default configuration file, overriding existing file
    #[clap(short, long)]
    force: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    args.configure()
}

impl Args {
    fn configure(&self) -> anyhow::Result<()> {
        let config_path = Config::path()?;

        if Path::new(&config_path).exists() && !self.force {
            Err(anyhow!(
                "Configuration file already exists, use --force to overwrite it"
            ))
        } else {
            Config::default(&config_path)?;
            crate::wout!("{} {}", SUCCESS, config_path);
            Ok(())
        }
    }
}
