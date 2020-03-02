use docopt::Docopt;
use prettytable::format::Alignment;
use prettytable::{color, Attr, Cell};
use rand::Rng;
use serde::de::DeserializeOwned;
use xdg::BaseDirectories;

use std::iter;

use crate::config::Config;
use crate::entity::money::{Currency, Money};
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
    std::env::var("EDITOR").map_err(|_| CliError::UndefinedEditor)
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

pub fn currency(value: &str, config: &Config) -> CliResult<Currency> {
    let currency_code = if value.is_empty() {
        config.currency.to_string()
    } else {
        value.to_string()
    };

    let code = currency_code.to_uppercase();

    match steel_cent::currency::with_code(&code) {
        Some(val) => Ok(val.into()),
        None => Err(CliError::IncorrectCurrencyCode { code }),
    }
}

pub fn money_cell(
    value: &Money,
    with_sign: bool,
    with_brackets: bool,
    alignment: Alignment,
) -> Cell {
    let mut rep = if value.zero() {
        format!("{}", value).to_string()
    } else if with_sign {
        format!("{}", value)[0..].to_string()
    } else {
        format!("{}", value)[1..].to_string()
    };

    if with_brackets {
        rep = format!("({})", rep);
    };

    Cell::new_align(&rep, alignment)
        .with_style(Attr::Bold)
        .with_style(color(value.cents() as f64))
}

pub fn percentage_cell(dividend: &Money, divisor: &Money, alignment: Alignment) -> Cell {
    let value = if divisor.zero() {
        -100.0
    } else {
        (dividend.cents() as f64 / divisor.cents() as f64) * 100.0
    };

    Cell::new_align(&format!("{:+.2}%", value)[1..], alignment)
        .with_style(Attr::Bold)
        .with_style(color(value))
}

fn color(value: f64) -> Attr {
    match value {
        v if v > 0.0 => Attr::ForegroundColor(color::BRIGHT_GREEN),
        v if v < 0.0 => Attr::ForegroundColor(color::BRIGHT_RED),
        _ => Attr::ForegroundColor(color::BRIGHT_BLACK),
    }
}
