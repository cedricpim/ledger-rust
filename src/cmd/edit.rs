use serde::Deserialize;

use std::process::Command;

use crate::CliResult;
use crate::CliError;
use crate::util;
use crate::config;


static USAGE: &'static str = "
Allows editing of the CSV (ledger or networth).

Sometimes the best way to do any changes to the CSV is by opening the preferred editor (defined on $EDITOR) and do the changes directly. This command does just that, while handling the decryption/encryption (if enabled).

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
    fn edit(&self, config: serde_yaml::Value) -> CliResult<()> {
        let editor = Args::editor()?;

        let mut filepath = self.filepath(config)?;

        if self.flag_line != 0 {
            let suffix = ":".to_owned() + &self.flag_line.to_string();
            filepath.push_str(&suffix);
        }

        Command::new(editor).arg(filepath).status()?;

        return Ok(());
    }

    fn filepath(&self, config: serde_yaml::Value) -> CliResult<String> {
        let key = if self.flag_networth { "networth" } else { "ledger" };

        return match config.get("file").and_then(|v| v.get(key)).and_then(|v| v.as_str()) {
            None => Err(CliError::from("Missing key 'ledger' on configuration file")),
            Some(val) => Ok(shellexpand::tilde(val).to_string())
        };
    }

    fn editor() -> CliResult<String> {
        return match option_env!("EDITOR") {
            None => Err(CliError::from("EDITOR variable is not set")),
            Some(val) => Ok(val.to_string())
        };
    }
}
