use serde::Deserialize;

use crate::{util, CliResult};

static USAGE: &str = "
Pull entries and transactions from Firefly III and then push local changes to Firefly III.

This command will first pull any new entries and transactions from Firefly III into local storage
and, only after, it will push the existing local changes to Firefly III (if the configuration file
is set for Firefly). This command also replaces the usage of push command since pushing entries
without first pulling any new entries could create problems in the system (since a new id for the
pushed local change would be stored locally). For setting up the configuration with Firefly, ensure
that the key \"firefly\" has a valid access token in the configuration file.

Usage:
    ledger sync [options]

Options:
    -h, --help          Display this message
";

#[derive(Debug, Deserialize)]
struct Args {}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let _args: Args = util::get_args(USAGE, argv)?;

    crate::cmd::pull::run(&[argv[0], &"pull"])?;
    crate::cmd::push::run(&[argv[0], &"push"])?;
    Ok(())
}
