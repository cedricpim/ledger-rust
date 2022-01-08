use clap::Clap;

use crate::config::Config;
use crate::entity::sync::pull::Pull;
use crate::CliResult;

static MISSING_KEY: &str = "There is no key set up";

#[derive(Clap, Debug, Default)]
pub struct Args {}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.pull(config)
}

impl Args {
    fn pull(&self, config: Config) -> CliResult<()> {
        match &config.firefly {
            Some(val) => Pull::new(val).perform(config),
            None => crate::werr!(2, "{}", MISSING_KEY),
        }
    }
}
