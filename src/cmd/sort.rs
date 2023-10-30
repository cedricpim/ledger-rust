use clap::Parser;

use crate::config::Config;
use crate::entity::line::Line;
use crate::resource::Resource;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(
        value_enum,
        default_value = "ledger",
        default_value_if("networth", "true", Some("networth")),
        hide = true
    )]
    mode: crate::Mode,
    /// Sort entries from networth CSV instead of ledger CSV
    #[arg(short, long)]
    networth: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;

    args.sort(&config)
}

impl Args {
    fn sort(&self, config: &Config) -> anyhow::Result<()> {
        let mut resource = Resource::new(config, self.mode)?;

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
