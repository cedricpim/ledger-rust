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

pub fn load() -> CliResult<Config> {
    let config_path = configuration()?;

    if !Path::new(&config_path).exists() {
        create_default_file(&config_path)?;
    };

    let file = File::open(&config_path)?;
    let data: Config = serde_yaml::from_reader(file)?;
    Ok(data)
}

pub fn configuration() -> CliResult<PathBuf> {
    let xdg_dirs = root()?;
    let config_path = xdg_dirs
        .place_config_file(CONFIGURATION_FILENAME)
        .map_err(CliError::from)?;
    Ok(config_path)
}

pub fn create_default_file(config_path: &PathBuf) -> CliResult<()> {
    let default = default()?;
    let mut file = File::create(&config_path)?;
    let yaml = serde_yaml::to_string(&default)?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}

fn root() -> CliResult<xdg::BaseDirectories> {
    xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).map_err(CliError::from)
}

fn default() -> CliResult<Config> {
    let mut config_dir = root()?.get_config_home();

    let default = Config {
        encryption: random_pass(),
        files: Files {
            ledger: default_filepath(&mut config_dir, "ledger.csv")?,
            networth: default_filepath(&mut config_dir, "networth.csv")?,
        },
    };

    Ok(default)
}

fn random_pass() -> Option<String> {
    let mut rng = rand::thread_rng();
    let chars: String = iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .take(32)
        .collect();

    Some(chars)
}

fn default_filepath(dir: &mut PathBuf, filename: &str) -> CliResult<String> {
    dir.push(filename);

    let filepath = dir
        .to_str()
        .map(|v| v.to_string())
        .ok_or(CliError::IncorrectPath {
            filename: filename.to_string(),
        });

    dir.pop();

    filepath
}
