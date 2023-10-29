use clap::{Parser, Subcommand, ValueEnum};

use std::process;

mod cmd;
mod config;
mod crypto;
mod entity;
mod exchange;
mod filter;
mod resource;
mod service;
mod util;
mod xdg;

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

#[derive(Parser, Debug)]
#[command(author, about, version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Calculate the current balances for each account
    ///
    /// This command will calculate the current balance of each account and
    /// display it.
    Balance(cmd::balance::Args),
    /// Add a line to the ledger or networth
    ///
    /// This command will, if used without any arguments, request all the
    /// fields that compose a single transaction/entry or create
    /// a transaction/entry based in the arguments provided. It will then
    /// store the transaction in the ledger file (or the entry in the
    /// networth file).
    Book(cmd::book::Args),

    /// Copy default configuration file to the default location
    ///
    /// In order to allow some flexibility to the application, there are
    /// some options that can be defined in a configuration file. To improve
    /// the usability, there is a default configuration file, properly
    /// commented, and this command copies it to the expected location.
    Configure(cmd::configure::Args),
    /// Convert other currencies to main currency of the account
    ///
    /// Since it isn't always possible to provide the correct currency of
    /// the money spent in each account, this option allows the user to
    /// provide entries in any currency and then, once this is run,
    /// converting all entries of a given account to the main currency of
    /// that account. The main currency of each account is calculated by
    /// checking the currency of the first transaction that occurred for
    /// each unique account.
    Convert(cmd::convert::Args),
    /// Create a new ledger/networth file
    ///
    /// This allows the initial set up of the main file that will be used to
    /// store either the transactions or the networth entries. If the file
    /// already exists, it won't be touched. The file will be created with the
    /// headers, and if encryption is set, it will also be encrypted.
    Create(cmd::create::Args),
    /// Open ledger/networth file in your editor
    ///
    /// Sometimes the best way to do any changes to the CSV is by opening the
    /// preferred editor (defined on $EDITOR) and do the changes directly.
    /// This command does just that, while handling the decryption/encryption
    /// (if enabled).
    Edit(cmd::edit::Args),
    /// Calculate current networth
    ///
    /// This command will print the list of the current networth, per asset.
    /// If the storage option is provided, then the total amount of the
    /// current networth is stored in the networth CSV as a new entry.
    Networth(cmd::networth::Args),
    /// Pull new changes from Firefly III
    ///
    /// This command will get the latest transaction locally (by id), and
    /// pull and the entries above that id from Firefly III (if the
    /// configuration file is set for Firefly). For setting up the
    /// configuration with Firefly, ensure that the key "firefly" has
    /// a valid access token in the configuration file.
    Pull(cmd::pull::Args),
    /// Sync new changes from and push local changes to Firefly III
    ///
    /// This command will first pull any new entries and transactions from
    /// Firefly III into local storage and, only after, it will push the
    /// existing local changes to Firefly III (if the configuration file is
    /// set for Firefly). This command also replaces the usage of push
    /// command since pushing entries without first pulling any new entries
    /// could create problems in the system (since a new id for the pushed
    /// local change would be stored locally). For setting up the
    /// configuration with Firefly, ensure that the key "firefly" has
    /// a valid access token in the configuration file.
    Sync(cmd::sync::Args),
    /// Create a report about the transactions on the ledger
    ///
    /// This command will generate a report, based on a defined time period,
    /// about all the transactions included in that time period. This report
    /// is shown in a single currency (all transactions that are not in this
    /// currency, are exchanged to it with the current rates) and there is
    /// no distinction made regarding different accounts - transactions are
    /// only aggregate per category.
    Report(cmd::report::Args),
    /// Display all transactions
    ///
    /// This command will generate a report, based on a defined time period,
    /// about all the transactions included in that time period. This report
    /// is shown in a single currency (all transactions that are not in this
    /// currency, are exchanged to it with the current rates) and there is
    /// no distinction made regarding different accounts - transactions are
    /// only aggregate per category.
    Show(cmd::show::Args),
    /// Sort the entries in the ledger.
    ///
    /// This command will rewrite the existing file, but sorted according to
    /// the date of each entry. Unless the date is different, the entries
    /// should remain unchanged (date is the only attribute used for sorting).
    Sort(cmd::sort::Args),
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Mode {
    Ledger,
    Networth,
}

fn main() {
    if std::env::var("DEBUG").is_ok() {
        std::env::set_var("RUST_LOG", "TRACE");
    }

    env_logger::init();

    let result = match App::parse().command {
        Commands::Balance(args) => cmd::balance::run(args),
        Commands::Book(args) => cmd::book::run(args),
        Commands::Edit(args) => cmd::edit::run(args),
        Commands::Configure(args) => cmd::configure::run(args),
        Commands::Convert(args) => cmd::convert::run(args),
        Commands::Create(args) => cmd::create::run(args),
        Commands::Networth(args) => cmd::networth::run(args),
        Commands::Pull(args) => cmd::pull::run(args),
        Commands::Sync(args) => cmd::sync::run(args),
        Commands::Report(args) => cmd::report::run(args),
        Commands::Show(args) => cmd::show::run(args),
        Commands::Sort(args) => cmd::sort::run(args),
    };

    match result {
        Ok(()) => process::exit(0),
        Err(err) => werr!(1, "{}", err),
    }
}
