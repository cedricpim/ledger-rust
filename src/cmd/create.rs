use clap::Clap;

use std::path::Path;

use crate::config::Config;
use crate::error::CliError;

use crate::resource::Resource;
use crate::CliResult;

static SUCCESS: &str = "Generated default file on";

#[derive(Clap, Debug)]
pub struct Args {
    #[clap(
        arg_enum,
        default_value = "ledger",
        default_value_if("networth", None, "networth"),
        hidden = true
    )]
    mode: crate::Mode,
    /// Create networth CSV instead of ledger CSV
    #[clap(short, long)]
    networth: bool,
    /// Create the initial file, overriding existing one
    #[clap(short, long)]
    force: bool,
}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.create(&config)
}

impl Args {
    fn create(&self, config: &Config) -> CliResult<()> {
        let resource = Resource::new(config, self.mode)?;

        if Path::new(&resource.filepath).exists() && !self.force {
            Err(CliError::ExistingFile {
                filepath: resource.filepath,
            })
        } else {
            resource.create()?;
            crate::wout!("{} {}", SUCCESS, resource.filepath);
            Ok(())
        }
    }
}
