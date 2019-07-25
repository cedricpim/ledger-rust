use docopt::Docopt;
use serde::Deserialize;

use std::env;
use std::fmt;
use std::process;

mod cmd;
mod util;

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

macro_rules! fail {
    ($e:expr) => {
        Err(::std::convert::From::from($e))
    };
}

macro_rules! command_list {
    () => (
"
    analysis    List all transactions on the ledger for the specified category
    balance     List the current balance of each account
    book        Add a transaction to the ledger
    compare     Compare multiple periods
    configure   Copy provided configuration file to the default location
    convert     Convert other currencies to main currency of the account
    create      Create a new ledger/networth file
    edit        Open ledger/networth file in your editor
    networth    Calculate current networth
    report      Create a report about the transactions on the ledger according to any params provided
    show        Display all transactions
    trip        Create a report about the trips present on the ledger
"
    )
}

static USAGE: &'static str = concat!(
    "
Usage:
    ledger <command> [<args>...] [options]
    ledger --list
    ledger --help
    ledger --version
Options:
    <command> -h  Display the command help message
Commands:",
    command_list!()
);

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: Option<Command>,
    flag_list: bool,
    flag_help: bool,
    flag_version: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Command {
    Edit,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.version(Some(util::version())).deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("{:?}", args);

    if args.flag_list {
        wout!(concat!("Installed commands:", command_list!()));
        return;
    }

    match args.arg_command {
        None => {
            werr!(concat!(
                "ledger is a command line tool for tracking expenses.

Please choose one of the following commands:",
                command_list!()
            ));
            process::exit(0);
        }
        Some(cmd) => match cmd.run() {
            Ok(()) => process::exit(0),
            Err(CliError::Flag(err)) => err.exit(),
            Err(CliError::Csv(err)) => {
                werr!("{}", err);
                process::exit(1);
            }
            Err(CliError::Other(msg)) => {
                werr!("{}", msg);
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
            return Err(CliError::Other(
                format!(
                    "ledger expects commands in lowercase. Did you mean '{}'?",
                    argv[1].to_lowercase()
                )
                .to_string(),
            ));
        }

        match self {
            Command::Edit => cmd::edit::run(argv),
        }
    }
}

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Flag(docopt::Error),
    Csv(csv::Error),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Flag(ref e) => e.fmt(f),
            CliError::Csv(ref e) => e.fmt(f),
            CliError::Other(ref s) => f.write_str(&**s),
        }
    }
}

impl From<docopt::Error> for CliError {
    fn from(err: docopt::Error) -> CliError {
        CliError::Flag(err)
    }
}

impl From<csv::Error> for CliError {
    fn from(err: csv::Error) -> CliError {
        CliError::Csv(err)
    }
}

impl From<String> for CliError {
    fn from(err: String) -> CliError {
        CliError::Other(err)
    }
}

impl<'a> From<&'a str> for CliError {
    fn from(err: &'a str) -> CliError {
        CliError::Other(err.to_owned())
    }
}
