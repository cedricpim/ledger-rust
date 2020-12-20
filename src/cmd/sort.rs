use clap::Clap;

use crate::config::Config;
use crate::entity::line::Line;
use crate::resource::Resource;
use crate::CliResult;

#[derive(Clap, Debug)]
pub struct Args {
    #[clap(
        arg_enum,
        default_value = "ledger",
        default_value_if("networth", None, "networth"),
        hidden = true
    )]
    mode: crate::Mode,
    /// Sort entries from networth CSV instead of ledger CSV
    #[clap(short, long)]
    networth: bool,
}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.sort(&config)
}

impl Args {
    fn sort(&self, config: &Config) -> CliResult<()> {
        let resource = Resource::new(&config, self.mode)?;

        let mut lines: Vec<Line> = Vec::new();

        resource.line(&mut |record| {
            lines.push(record.clone());
            Ok(())
        })?;

        lines.sort();

        resource.create_with(lines)?;

        Ok(())
    }
}
