use anyhow::anyhow;
use xdg::BaseDirectories;

use std::fmt;

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
    pub fn filepath(&self) -> anyhow::Result<String> {
        let directory = Self::directory()?;

        let filepath = match self {
            Xdg::Config(filename) => directory.place_config_file(filename)?,
            Xdg::Cache(filename) => directory.place_cache_file(filename)?,
        };

        filepath.to_str().map(|v| v.to_string()).ok_or(anyhow!(
            "An error occurred while determining the path for: {}",
            self.to_string()
        ))
    }

    fn directory() -> anyhow::Result<BaseDirectories> {
        Ok(BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))?)
    }
}
