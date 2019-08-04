use docopt::Docopt;
use serde::de::DeserializeOwned;

use crate::CliResult;
use crate::error::CliError;

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
        where T: DeserializeOwned {
            Docopt::new(usage)
                .and_then(|d| d.argv(argv.iter().map(|&x| x))
                                .version(Some(version()))
                                .deserialize())
                .map_err(From::from)
}

pub fn editor() -> CliResult<String> {
    return match option_env!("EDITOR") {
        None => Err(CliError::UndefinedEditor),
        Some(val) => Ok(val.to_string()),
    };
}
