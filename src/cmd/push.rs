use clap::Parser;

use crate::config::Config;
use crate::entity::sync::push::Push;

static MISSING_KEY: &str = "There is no key set up";

#[derive(Parser, Debug, Default)]
pub struct Args {}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.push(config)
}

impl Args {
    fn push(&self, config: Config) -> anyhow::Result<()> {
        match &config.firefly {
            Some(val) => Push::new(val, &config)?.perform(config),
            None => crate::werr!(2, "{}", MISSING_KEY),
        }
    }
}
