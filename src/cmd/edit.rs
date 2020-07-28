use serde::Deserialize;

use std::process::Command;

use crate::config::Config;
use crate::resource::Resource;
use crate::{util, CliResult};

static USAGE: &str = "
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
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.edit(&config)
}

impl Args {
    // After manual changes, validate the entries by loading all the records. This is done after
    // the file is saved so that errors can be fixed and all the data already input is not lost.
    fn edit(&self, config: &Config) -> CliResult<()> {
        let editor = util::editor()?;
        let resource = Resource::new(&config, self.flag_networth)?;

        resource.apply(|file| {
            let filepath = self.filepath(file.path().display());
            Command::new(editor).arg(filepath).status()?;
            Ok(())
        })?;

        resource.line(&mut |_record| Ok(()))
    }

    fn filepath(&self, filepath: std::path::Display) -> String {
        if self.flag_line == 0 {
            format!("{}", filepath)
        } else {
            format!("{}:{}", filepath, self.flag_line)
        }
    }
}
