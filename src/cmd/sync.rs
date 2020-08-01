use clap::Clap;

use crate::cmd::{pull, push};
use crate::CliResult;

#[derive(Clap, Debug)]
pub struct Args {}

pub fn run(_args: Args) -> CliResult<()> {
    pull::run(pull::Args::default())?;
    push::run(push::Args::default())?;

    Ok(())
}
