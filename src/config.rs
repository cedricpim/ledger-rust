use xdg;
use serde_yaml;

use std::io;
use std::path::{Path, PathBuf};

use crate::CliResult;
use crate::CliError;

pub fn load() -> CliResult<serde_yaml::Value> {
    let config_path = configuration()?;

    if !Path::new(&config_path).exists() {
        return Err(CliError::from("Configuration file does not exist"));
    }

    let file = std::fs::File::open(config_path)?;
    let data: serde_yaml::Value = serde_yaml::from_reader(file)?;
    return Ok(data);
}

fn configuration() -> io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("ledger").unwrap();
    let config_path = xdg_dirs.place_config_file("rust-config")?;
    return Ok(config_path);
}
