use docopt::Docopt;
use rand::Rng;
use serde::de::DeserializeOwned;
use xdg::BaseDirectories;

use std::iter;

use crate::error::CliError;
use crate::CliResult;

pub fn version() -> String {
    let (maj, min, pat) = (
        option_env!("CARGO_PKG_VERSION_MAJOR"),
        option_env!("CARGO_PKG_VERSION_MINOR"),
        option_env!("CARGO_PKG_VERSION_PATCH"),
    );
    match (maj, min, pat) {
        (Some(maj), Some(min), Some(pat)) => format!("{}.{}.{}", maj, min, pat),
        _ => "".to_owned(),
    }
}

pub fn get_args<T>(usage: &str, argv: &[&str]) -> CliResult<T>
where
    T: DeserializeOwned,
{
    Docopt::new(usage)
        .and_then(|d| {
            d.argv(argv.iter().copied())
                .version(Some(version()))
                .deserialize()
        })
        .map_err(From::from)
}

pub fn editor() -> CliResult<String> {
    match option_env!("EDITOR") {
        None => Err(CliError::UndefinedEditor),
        Some(val) => Ok(val.to_string()),
    }
}

pub fn main_directory() -> CliResult<BaseDirectories> {
    BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).map_err(CliError::from)
}

pub fn random_pass() -> Option<String> {
    let mut rng = rand::thread_rng();
    let chars: String = iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .take(32)
        .collect();

    Some(chars)
}

pub fn config_filepath(filename: &str) -> CliResult<String> {
    let dir = main_directory()?
        .place_config_file(filename)
        .map_err(CliError::from)?;

    dir.to_str()
        .map(|v| v.to_string())
        .ok_or(CliError::IncorrectPath {
            filename: filename.to_string(),
        })
}
