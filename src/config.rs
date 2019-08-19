use serde_yaml;

use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::{util, CliResult};

const CONFIGURATION_FILENAME: &str = "rust-config";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    encryption: Option<String>,
    files: Files,
}

#[derive(Debug, Serialize, Deserialize)]
struct Files {
    ledger: String,
    networth: String,
}

impl Config {
    pub fn new() -> CliResult<Config> {
        let config_path = Config::path()?;

        let data: Config = if Path::new(&config_path).exists() {
            let file = File::open(&config_path)?;
            serde_yaml::from_reader(file)?
        } else {
            Config::default(&config_path)?
        };

        Ok(data)
    }

    pub fn default(config_path: &str) -> CliResult<Config> {
        let default = Config {
            encryption: util::random_pass(),
            files: Files {
                ledger: util::config_filepath("ledger.csv")?,
                networth: util::config_filepath("networth.csv")?,
            },
        };

        let mut file = File::create(&config_path)?;
        let yaml = serde_yaml::to_string(&default)?;
        file.write_all(yaml.as_bytes())?;
        Ok(default)
    }

    pub fn path() -> CliResult<String> {
        util::config_filepath(CONFIGURATION_FILENAME)
    }

    pub fn filepath(&self, networth: bool) -> String {
        let val = if networth {
            &self.files.networth
        } else {
            &self.files.ledger
        };

        shellexpand::tilde(val).to_string()
    }

    pub fn pass(&self) -> Option<String> {
        self.encryption.to_owned()
    }
}
