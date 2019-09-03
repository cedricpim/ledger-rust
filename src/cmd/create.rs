use serde::Deserialize;

use std::path::Path;

use crate::config::Config;
use crate::error::CliError;
use crate::{repository, util, CliResult};

static USAGE: &'static str = "
Creates the ledger or networth file that will be used to store the entries.

This allows the initial set up of the main file that will be used to store either the transactions
or the networth entries. If the file already exists, it won't be touched. The file will be
created with the headers, and if encryption is set, it will also be encrypted.

Usage:
    ledger create [options]

Options:
    -n, --networth      Create networth CSV instead of ledger CSV
    -f, --force         Create the initial file, overriding existing one
    -h, --help          Display this message
";

static SUCCESS: &'static str = "Generated default file on";

#[derive(Debug, Deserialize)]
struct Args {
    flag_networth: bool,
    flag_force: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.create(&config)
}

impl Args {
    fn create(&self, config: &Config) -> CliResult<()> {
        let resource = repository::Resource::new(&config, self.flag_networth)?;

        if Path::new(&resource.filepath).exists() && !self.flag_force {
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
