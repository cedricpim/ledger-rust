use clap::Parser;

use std::process::Command;

use crate::config::Config;
use crate::resource::Resource;
use crate::{util, CliResult};

static VIM: &str = "vim";

#[derive(Parser, Debug)]
pub struct Args {
    /// Open file with cursor in the last line (Only supported for vim and variants)
    #[clap(short, long)]
    bottom: bool,
    #[clap(
        arg_enum,
        default_value = "ledger",
        default_value_if("networth", None, Some("networth")),
        hide = true
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
        let mut resource = Resource::new(config, self.mode)?;

        resource.apply(|file| {
            let arguments = self.arguments(&editor, file.path().display());
            Command::new(editor).args(arguments).status()?;
            Ok(())
        })?;

        resource.line(&mut |_record| Ok(()))
    }

    fn arguments(&self, editor: &str, filepath: std::path::Display) -> Vec<String> {
        if self.bottom && editor.contains(VIM) {
            vec!["+".to_string(), filepath.to_string()]
        } else {
            vec![filepath.to_string()]
        }
    }
}
