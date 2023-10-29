use clap::Parser;

use crate::config::Config;
use crate::entity::sync::pull::Pull;

static MISSING_KEY: &str = "There is no key set up";

#[derive(Parser, Debug, Default)]
pub struct Args {}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.pull(config)
}

impl Args {
    fn pull(&self, config: Config) -> anyhow::Result<()> {
        match &config.firefly {
            Some(val) => Pull::new(val).perform(config),
            None => crate::werr!(2, "{}", MISSING_KEY),
        }
    }
}
