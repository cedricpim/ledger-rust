use serde_yaml;

use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::error::CliError;
use crate::{util, CliResult};

const CONFIGURATION_FILENAME: &str = "rust-config";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    encryption: Option<String>,
    files: Files,
    exchange: Exchange,
    pub ignored_categories: Vec<String>,
    pub ignored_accounts: Vec<String>,
    pub investments: Vec<String>,
    pub currency: String,
    pub firefly: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct Files {
    ledger: String,
    networth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
    api_key: String,
    cache_file: String,
    ttl: u64,
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
            exchange: Exchange {
                api_key: "your app id from https://openexchangerates.org/signup".to_string(),
                cache_file: "/tmp/exchange-cache-rust.yml".to_string(),
                ttl: 86400, // 1 day
            },
            currency: "EUR".to_string(),
            ignored_categories: vec!["Investment".to_string()],
            ignored_accounts: vec!["Personal".to_string()],
            investments: vec!["Investment".to_string()],
            firefly: None,
        };

        let mut file = File::create(&config_path)?;
        let yaml = serde_yaml::to_string(&default)?;
        file.write_all(yaml.as_bytes())?;
        Ok(default)
    }

    pub fn path() -> CliResult<String> {
        util::config_filepath(CONFIGURATION_FILENAME)
    }

    pub fn exchange(&self) -> Exchange {
        self.exchange.clone()
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

impl Exchange {
    pub fn cached(&self) -> bool {
        let path = Path::new(&self.cache_file);

        let mtime = path
            .metadata()
            .and_then(|v| v.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let default = Duration::new(self.ttl, 0);

        let interval = SystemTime::now()
            .duration_since(mtime)
            .unwrap_or(default)
            .as_secs();

        path.exists() && interval < self.ttl
    }

    pub fn open(&self) -> CliResult<File> {
        File::open(&self.cache_file).map_err(CliError::from)
    }

    pub fn create(&self) -> CliResult<File> {
        File::create(&self.cache_file).map_err(CliError::from)
    }

    pub fn key(&self) -> String {
        self.api_key.to_owned()
    }
}
