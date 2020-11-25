use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::entity::line::Liner;
use crate::resource::Resource;
use crate::xdg::Xdg;
use crate::{util, CliResult, Mode};

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
    pub firefly: Option<FireflyOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Files {
    ledger: String,
    networth: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FireflyOptions {
    pub base_path: String,
    pub token: String,
    pub opening_balance: String,
    #[serde(skip)]
    pub currency: String,
    #[serde(skip)]
    pub transfer: String,
}

impl FireflyOptions {
    pub fn build(firefly_options: &FireflyOptions, config: &Config) -> Self {
        Self {
            base_path: firefly_options.base_path.to_string(),
            token: firefly_options.token.to_string(),
            opening_balance: firefly_options.opening_balance.to_string(),
            currency: config.currency.to_string(),
            transfer: config.transfer.to_string(),
        }
    }
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
                ledger: Xdg::Config("ledger.csv".to_string()).filepath()?,
                networth: Xdg::Config("networth.csv".to_string()).filepath()?,
            },
            exchange_key: "your app id from https://openexchangerates.org/signup".to_string(),
            currency: "EUR".to_string(),
            transfer: "Transfer".to_string(),
            ignored_accounts: vec!["Personal".to_string()],
            investments: "Investment".to_string(),
            firefly: None,
        };

        let mut file = File::create(&config_path)?;
        let yaml = serde_yaml::to_string(&default)?;
        file.write_all(yaml.as_bytes())?;
        Ok(default)
    }

    pub fn path() -> CliResult<String> {
        Xdg::Config(CONFIGURATION_FILENAME.to_string()).filepath()
    }

    pub fn filepath(&self, mode: Mode) -> String {
        let path = match mode {
            Mode::Ledger => &self.files.ledger,
            Mode::Networth => &self.files.networth,
        };

        shellexpand::tilde(path).to_string()
    }

    pub fn pass(&self) -> Option<String> {
        self.encryption.to_owned()
    }

    pub fn exchange_key(&self) -> String {
        self.exchange_key.to_owned()
    }

    pub fn total_pushable_lines(&self) -> CliResult<usize> {
        let mut ledger_lines = 0;
        Resource::new(&self, Mode::Ledger)?.line(&mut |record| {
            if record.pushable() {
                ledger_lines += 1
            };
            Ok(())
        })?;

        let mut networth_lines = 0;
        Resource::new(&self, Mode::Networth)?.line(&mut |record| {
            if record.pushable() {
                networth_lines += 1
            };
            Ok(())
        })?;

        Ok(networth_lines + ledger_lines)
    }
}
