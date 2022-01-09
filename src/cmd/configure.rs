use clap::Parser;

use std::path::Path;

use crate::config::Config;
use crate::error::CliError;
use crate::CliResult;

static SUCCESS: &str = "Generated default configuration file on";

#[derive(Parser, Debug)]
pub struct Args {
    /// Copy the default configuration file, overriding existing file
    #[clap(short, long)]
    force: bool,
}

pub fn run(args: Args) -> CliResult<()> {
    args.configure()
}

impl Args {
    fn configure(&self) -> CliResult<()> {
        let config_path = Config::path()?;

        if Path::new(&config_path).exists() && !self.force {
            Err(CliError::ExistingConfiguration)
        } else {
            Config::default(&config_path)?;
            crate::wout!("{} {}", SUCCESS, config_path);
            Ok(())
        }
    }
}
