use serde::Deserialize;

use std::process::Command;

use crate::{config,util,CliResult};

static USAGE: &'static str = "
Allows editing of the CSV (ledger or networth).

Sometimes the best way to do any changes to the CSV is by opening the preferred editor (defined on
$EDITOR) and do the changes directly. This command does just that, while handling the
decryption/encryption (if enabled).

Usage:
    ledger edit [options]

Options:
    -l, --line=<line>   Line in which to open the file
    -n, --networth      Open networth CSV instead of ledger CSV
    -h, --help          Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_line: i32,
    flag_networth: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let config = config::load()?;

    let args: Args = util::get_args(USAGE, argv)?;

    return args.edit(config);
}

impl Args {
    fn edit(&self, config: config::Config) -> CliResult<()> {
        let editor = util::editor()?;
        let filepath = self.filepath(config)?;

        Command::new(editor).arg(filepath).status()?;

        return Ok(());
    }

    fn filepath(&self, config: config::Config) -> CliResult<String> {
        return if self.flag_line == 0 {
            config.filepath(self.flag_networth)
        } else {
            Ok(format!("{}:{}", config.filepath(self.flag_networth)?, self.flag_line))
        };
    }
}
