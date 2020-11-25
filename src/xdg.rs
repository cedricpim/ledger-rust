use std::fmt;

use xdg::BaseDirectories;

use crate::error::CliError;
use crate::CliResult;

#[derive(Debug)]
pub enum Xdg {
    Config(String),
    Cache(String),
}

impl fmt::Display for Xdg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Xdg::Config(filename) => write!(f, "config({:?})", filename),
            Xdg::Cache(filename) => write!(f, "cache({:?})", filename),
        }
    }
}

impl Xdg {
    pub fn filepath(&self) -> CliResult<String> {
        let directory = Self::directory()?;

        let filepath = match self {
            Xdg::Config(filename) => directory.place_config_file(filename)?,
            Xdg::Cache(filename) => directory.place_cache_file(filename)?,
        };

        filepath
            .to_str()
            .map(|v| v.to_string())
            .ok_or(CliError::IncorrectPath {
                message: self.to_string(),
            })
    }

    fn directory() -> CliResult<BaseDirectories> {
        BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).map_err(CliError::from)
    }
}
