use docopt::Docopt;
use serde::Deserialize;

use std::env;
use std::fmt;
use std::io;
use std::process;

mod cmd;
mod config;
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
    ledger <command> [<args>...]
    ledger [options]

Options:
    -l, --list      List commands
    -h, --help      Display this message
    -v, --version   Print version info and exit

Commands:",
    command_list!()
);

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: Option<Command>,
    flag_list: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Command {
    Edit,
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
            werr!("ledger: try 'ledger --help' for more information");
            process::exit(2);
        }
        Some(cmd) => match cmd.run() {
            Ok(()) => process::exit(0),
            Err(CliError::Flag(err)) => err.exit(),
            Err(CliError::Csv(err)) => {
                werr!("{}", err);
                process::exit(1);
            }
            Err(CliError::Io(err)) => {
                werr!("{}", err);
                process::exit(1);
            }
            Err(CliError::Yaml(err)) => {
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

        return match self {
            Command::Edit => cmd::edit::run(argv),
        };
    }
}

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Flag(docopt::Error),
    Csv(csv::Error),
    Io(io::Error),
    Yaml(serde_yaml::Error),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Flag(ref e) => e.fmt(f),
            CliError::Csv(ref e) => e.fmt(f),
            CliError::Io(ref e) => e.fmt(f),
            CliError::Yaml(ref e) => e.fmt(f),
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

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<serde_yaml::Error> for CliError {
    fn from(err: serde_yaml::Error) -> CliError {
        CliError::Yaml(err)
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
