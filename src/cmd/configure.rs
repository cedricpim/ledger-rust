use serde::Deserialize;

use std::io::Write;
use std::path::Path;

use crate::error::CliError;
use crate::{config, util, CliResult};

static USAGE: &'static str = "
Copies the default configuration file for the application.

In order to allow some flexibility to the application, there are some options that can be defined
in a configuration file. To improve the usability, there is a default configuration file, properly
commented, and this command copies it to the expected location.

Usage:
    ledger configure [options]

Options:
    -f, --force  Copy the default configuration file, overriding existing file
    -h, --help   Display this message
";

static SUCCESS: &'static str = "Generated default configuration file on";

#[derive(Debug, Deserialize)]
struct Args {
    flag_force: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    args.configure()
}

impl Args {
    pub fn configure(&self) -> CliResult<()> {
        let config_path = config::configuration()?;

        if Path::new(&config_path).exists() && !self.flag_force {
            Err(CliError::ExistingConfiguration)
        } else {
            config::create_default_file(&config_path)?;
            writeln!(
                &mut ::std::io::stdout(),
                "{} {}.",
                SUCCESS,
                config_path.as_path().display()
            )?;
            Ok(())
        }
    }
}
