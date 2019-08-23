use docopt::Docopt;
use serde::Deserialize;

#[macro_use]
extern crate derivative;

use std::{env, process};

mod cmd;
mod config;
mod crypto;
mod error;
mod filter;
mod line;
mod repository;
mod util;

#[macro_export]
macro_rules! wout {
    ($($arg:tt)*) => ({
        use std::io::Write;
        (writeln!(&mut ::std::io::stdout(), $($arg)*)).unwrap();
    });
}

macro_rules! werr {
    ($($arg:tt)*) => ({
        use std::io::Write;
        (writeln!(&mut ::std::io::stderr(), $($arg)*)).unwrap();
    });
}

macro_rules! command_list {
    () => (
"
Implemented:
    book        Add a transaction to the ledger
    configure   Copy provided configuration file to the default location
    create      Create a new ledger/networth file
    edit        Open ledger/networth file in your editor
    show        Display all transactions

To be implemented:
    analysis    List all transactions on the ledger for the specified category
    balance     List the current balance of each account
    compare     Compare multiple periods
    convert     Convert other currencies to main currency of the account
    networth    Calculate current networth
    report      Create a report about the transactions on the ledger according to any params provided
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
    Book,
    Configure,
    Create,
    Edit,
    Show,
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
        return wout!(concat!("Installed commands:", command_list!()));
    }

    match args.arg_command {
        None => {
            werr!("{}", EXECUTABLE);
            process::exit(2);
        }
        Some(cmd) => match cmd.run() {
            Ok(()) => process::exit(0),
            Err(err) => {
                werr!("{}", err);
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
            Command::Book => cmd::book::run(argv),
            Command::Edit => cmd::edit::run(argv),
            Command::Configure => cmd::configure::run(argv),
            Command::Create => cmd::create::run(argv),
            Command::Show => cmd::show::run(argv),
        }
    }
}
