use docopt::Docopt;
use serde::Deserialize;

use std::io::Write;
use std::{env, process};

mod cmd;
mod config;
mod crypto;
mod error;
mod repository;
mod util;

macro_rules! command_list {
    () => (
"
Implemented:
    configure   Copy provided configuration file to the default location
    edit        Open ledger/networth file in your editor

To be implemented:
    analysis    List all transactions on the ledger for the specified category
    balance     List the current balance of each account
    book        Add a transaction to the ledger
    compare     Compare multiple periods
    convert     Convert other currencies to main currency of the account
    create      Create a new ledger/networth file
    networth    Calculate current networth
    report      Create a report about the transactions on the ledger according to any params provided
    show        Display all transactions
    trip        Create a report about the trips present on the ledger
"
    )
}

static EXECUTABLE: &'static str = "ledger: try 'ledger --help' for more information";

static USAGE: &'static str = concat!(
    "
Usage:
    ledger <command> [<args>...]
    ledger [options]

Options:
    -l, --list      List commands
    -h, --help      Display this message
    -v, --version   Print version info and exit

Commands:",
    command_list!()
);

pub type CliResult<T> = Result<T, error::CliError>;

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: Option<Command>,
    flag_list: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Command {
    Edit,
    Configure,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| {
            d.options_first(true)
                .version(Some(util::version()))
                .deserialize()
        })
        .unwrap_or_else(|e| e.exit());

    if args.flag_list {
        return writeln!(
            &mut ::std::io::stdout(),
            concat!("Installed commands:", command_list!())
        )
        .unwrap();
    }

    match args.arg_command {
        None => {
            writeln!(&mut ::std::io::stderr(), "{}", EXECUTABLE).unwrap();
            process::exit(2);
        }
        Some(cmd) => match cmd.run() {
            Ok(()) => process::exit(0),
            Err(err) => {
                writeln!(&mut ::std::io::stderr(), "{}", err).unwrap();
                process::exit(1);
            }
        },
    }
}

impl Command {
    fn run(self) -> CliResult<()> {
        let argv: Vec<_> = env::args().map(|v| v.to_owned()).collect();
        let argv: Vec<_> = argv.iter().map(|s| &**s).collect();
        let argv = &*argv;

        if !argv[1].chars().all(char::is_lowercase) {
            return Err(error::CliError::InvalidCommand {
                command: argv[1].to_lowercase(),
            });
        }

        match self {
            Command::Edit => cmd::edit::run(argv),
            Command::Configure => cmd::configure::run(argv),
        }
    }
}
