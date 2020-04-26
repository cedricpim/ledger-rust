use docopt::Docopt;
use serde::Deserialize;

#[macro_use]
extern crate lazy_static;

use std::{env, process};

mod cmd;
mod config;
mod crypto;
mod entity;
mod error;
mod exchange;
mod filter;
mod repository;
mod service;
mod util;

#[macro_export]
macro_rules! wout {
    ($($arg:tt)*) => ({
        use std::io::Write;

        (writeln!(&mut ::std::io::stdout(), $($arg)*)).unwrap();
    });
}

#[macro_export]
macro_rules! werr {
    ($signal:tt, $($arg:tt)*) => ({
        use std::io::Write;
        use std::process;

        (writeln!(&mut ::std::io::stderr(), $($arg)*)).unwrap();
        process::exit($signal);
    });
}

macro_rules! command_list {
    () => (
"
Implemented:
    balance     List the current balance of each account
    book        Add a transaction to the ledger
    configure   Copy provided configuration file to the default location
    convert     Convert other currencies to main currency of the account
    create      Create a new ledger/networth file
    edit        Open ledger/networth file in your editor
    networth    Calculate current networth
    push        Push local changes to Firefly III
    report      Create a report about the transactions on the ledger according to any params provided
    show        Display all transactions
"
    )
}

static EXECUTABLE: &str = "ledger: try 'ledger --help' for more information";

static USAGE: &str = concat!(
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
    Balance,
    Book,
    Configure,
    Convert,
    Create,
    Edit,
    Networth,
    Push,
    Report,
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
        None => werr!(2, "{}", EXECUTABLE),
        Some(cmd) => match cmd.run() {
            Ok(()) => process::exit(0),
            Err(err) => werr!(1, "{}", err),
        },
    }
}

impl Command {
    fn run(self) -> CliResult<()> {
        let argv: Vec<_> = env::args().map(|v| v).collect();
        let argv: Vec<_> = argv.iter().map(|s| &**s).collect();
        let argv = &*argv;

        if !argv[1].chars().all(char::is_lowercase) {
            return Err(error::CliError::InvalidCommand {
                command: argv[1].to_lowercase(),
            });
        }

        match self {
            Command::Balance => cmd::balance::run(argv),
            Command::Book => cmd::book::run(argv),
            Command::Edit => cmd::edit::run(argv),
            Command::Configure => cmd::configure::run(argv),
            Command::Convert => cmd::convert::run(argv),
            Command::Create => cmd::create::run(argv),
            Command::Networth => cmd::networth::run(argv),
            Command::Push => cmd::push::run(argv),
            Command::Report => cmd::report::run(argv),
            Command::Show => cmd::show::run(argv),
        }
    }
}
