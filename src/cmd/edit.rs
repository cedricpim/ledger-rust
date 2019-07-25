use serde::Deserialize;

use std::process::Command;

use crate::CliResult;
use crate::util;

use crate::CliError;


static USAGE: &'static str = "
Allows editing of the CSV (ledger or networth).

Sometimes the best way to do any changes to the CSV is by opening the preferred editor (defined on $EDITOR) and do the changes directly. This command does just that, while handling the decryption/encryption (if enabled).

Usage:
    ledger edit [options]
    ledger edit --help

edit options:
    -l, --line      Line in which to open the file
    -n, --networth  Open networth CSV instead of ledger CSV
";

#[derive(Deserialize)]
struct Args {
    flag_line: i32,
    flag_networth: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    args.edit()
}

impl Args {
    fn edit(&self) -> CliResult<()> {
        let editor = match option_env!("EDITOR") {
            Some(val) => val,
            None => ""
        };

        Command::new(editor).arg("~/a").status();

        Ok(())
    }
}
