use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::xdg::Xdg;
use crate::{util, Mode};

const CONFIGURATION_FILENAME: &str = "config";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    encryption: Option<String>,
    files: Files,
    exchange_key: String,
    pub transfer: String,
    pub ignored_accounts: Vec<String>,
    pub investments: String,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Files {
    ledger: String,
    networth: String,
}

impl Config {
    pub fn new() -> anyhow::Result<Config> {
        let config_path = Config::path()?;

        let data: Config = if Path::new(&config_path).exists() {
            let file = File::open(&config_path)?;
            serde_yaml::from_reader(file)?
        } else {
            Config::default(&config_path)?
        };

        Ok(data)
    }

    pub fn default(config_path: &str) -> anyhow::Result<Config> {
        let default = Config {
            encryption: util::random_pass(),
            files: Files {
                ledger: Xdg::Config("ledger.csv".to_string()).filepath()?,
                networth: Xdg::Config("networth.csv".to_string()).filepath()?,
            },
            exchange_key: "your app id from https://openexchangerates.org/signup".to_string(),
            currency: "EUR".to_string(),
            transfer: "Transfer".to_string(),
            ignored_accounts: vec!["Personal".to_string()],
            investments: "Investment".to_string(),
        };

        let mut file = File::create(config_path)?;
        let yaml = serde_yaml::to_string(&default)?;
        file.write_all(yaml.as_bytes())?;
        Ok(default)
    }

    pub fn path() -> anyhow::Result<String> {
        Xdg::Config(CONFIGURATION_FILENAME.to_string()).filepath()
    }

    pub fn filepath(&self, mode: Mode) -> String {
        let path = match std::env::var("LEDGER_PATH") {
            Ok(val) => val,
            Err(_) => match mode {
                Mode::Ledger => self.files.ledger.to_string(),
                Mode::Networth => self.files.networth.to_string(),
            },
        };

        shellexpand::tilde(&path).to_string()
    }

    pub fn pass(&self) -> Option<String> {
        self.encryption.to_owned()
    }

    pub fn exchange_key(&self) -> String {
        self.exchange_key.to_owned()
    }
}
