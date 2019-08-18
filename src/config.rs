use serde_yaml;
use xdg;

use rand::Rng;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;
use std::iter;
use std::path::{Path, PathBuf};

use crate::error::CliError;
use crate::CliResult;

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
        let config_path = Config::location()?;

        let data: Config = if Path::new(&config_path).exists() {
            let file = File::open(&config_path)?;
            serde_yaml::from_reader(file)?
        } else {
            Config::default(&config_path)?
        };

        Ok(data)
    }

    pub fn default(config_path: &PathBuf) -> CliResult<Config> {
        let default = Config {
            encryption: Config::random_pass(),
            files: Files {
                ledger: Config::default_filepath("ledger.csv")?,
                networth: Config::default_filepath("networth.csv")?,
            },
        };

        let mut file = File::create(&config_path)?;
        let yaml = serde_yaml::to_string(&default)?;
        file.write_all(yaml.as_bytes())?;
        Ok(default)
    }

    pub fn location() -> CliResult<PathBuf> {
        let xdg_dirs = Config::root()?;
        let config_path = xdg_dirs
            .place_config_file(CONFIGURATION_FILENAME)
            .map_err(CliError::from)?;
        Ok(config_path)
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

    fn root() -> CliResult<xdg::BaseDirectories> {
        xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).map_err(CliError::from)
    }

    fn random_pass() -> Option<String> {
        let mut rng = rand::thread_rng();
        let chars: String = iter::repeat(())
            .map(|()| rng.sample(rand::distributions::Alphanumeric))
            .take(32)
            .collect();

        Some(chars)
    }

    fn default_filepath(filename: &str) -> CliResult<String> {
        let dir = Config::root()?.place_config_file(filename).map_err(CliError::from)?;

        dir.to_str()
            .map(|v| v.to_string())
            .ok_or(CliError::IncorrectPath {
                filename: filename.to_string(),
            })
    }
}
