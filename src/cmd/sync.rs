use clap::Parser;

use crate::cmd::{pull, push};

#[derive(Parser, Debug)]
pub struct Args {}

pub fn run(_args: Args) -> anyhow::Result<()> {
    pull::run(pull::Args::default())?;
    push::run(push::Args::default())?;

    Ok(())
}
