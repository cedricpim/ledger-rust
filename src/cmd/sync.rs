use clap::Parser;

use crate::cmd::{pull, push};
use crate::CliResult;

#[derive(Parser, Debug)]
pub struct Args {}

pub fn run(_args: Args) -> CliResult<()> {
    pull::run(pull::Args::default())?;
    push::run(push::Args::default())?;

    Ok(())
}
