use clap::Clap;

use crate::config::Config;
use crate::entity::sync::push::Push;
use crate::CliResult;

static MISSING_KEY: &str = "There is no key set up";

#[derive(Clap, Debug, Default)]
pub struct Args {}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.push(config)
}

impl Args {
    fn push(&self, config: Config) -> CliResult<()> {
        match &config.firefly {
            Some(val) => Push::new(&val, &config)?.perform(config),
            None => crate::werr!(2, "{}", MISSING_KEY),
        }
    }
}
