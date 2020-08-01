use clap::Clap;

use std::process::Command;

use crate::config::Config;
use crate::resource::Resource;
use crate::{util, CliResult};

#[derive(Clap, Debug)]
pub struct Args {
    /// Line in which to open the file
    #[clap(short, long, default_value = "1")]
    line: u32,
    #[clap(
        arg_enum,
        default_value = "ledger",
        default_value_if("networth", None, "networth"),
        hidden = true
    )]
    mode: crate::Mode,
    /// Open networth CSV instead of ledger CSV
    #[clap(short, long)]
    networth: bool,
}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.edit(&config)
}

impl Args {
    // After manual changes, validate the entries by loading all the records. This is done after
    // the file is saved so that errors can be fixed and all the data already input is not lost.
    fn edit(&self, config: &Config) -> CliResult<()> {
        let editor = util::editor()?;
        let resource = Resource::new(&config, self.mode)?;

        resource.apply(|file| {
            let filepath = format!("{}:{}", file.path().display(), self.line);
            Command::new(editor).arg(filepath).status()?;
            Ok(())
        })?;

        resource.line(&mut |_record| Ok(()))
    }
}
