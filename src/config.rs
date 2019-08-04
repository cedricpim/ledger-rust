use serde_yaml;
use xdg;

use std::fs::File;
use std::path::{Path, PathBuf};

use crate::CliResult;
use crate::error::CliError;

const CONFIGURATION_FILENAME: &'static str = "rust-config";

#[derive(Debug)]
pub struct Config {
    data: serde_yaml::Value,
}

impl Config {
    pub fn filepath(&self, networth: bool) -> CliResult<String> {
        let key = if networth {
            "networth"
        } else {
            "ledger"
        };

        return match self
            .data
            .get("file")
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_str())
        {
            None => Err(CliError::MissingFile {
                file: key.to_string(),
            }),
            Some(val) => Ok(shellexpand::tilde(val).to_string()),
        };
    }

    pub fn pass(&self) -> Option<String> {
        return self
            .data
            .get("encryption")
            .and_then(|v| v.as_str())
            .and_then(|v| Some(v.to_string()));
    }
}

pub fn load() -> CliResult<Config> {
    let config_path = configuration()?;

    if !Path::new(&config_path).exists() {
        return Err(CliError::MissingConfiguration);
    };

    let file = File::open(config_path)?;
    let data: serde_yaml::Value = serde_yaml::from_reader(file)?;
    return Ok(Config { data });
}

fn configuration() -> std::io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).unwrap();
    let config_path = xdg_dirs.place_config_file(CONFIGURATION_FILENAME)?;
    return Ok(config_path);
}
