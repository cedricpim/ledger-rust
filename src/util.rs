use prettytable::format::Alignment;
use prettytable::{color, Attr, Cell};
use rand::Rng;

use std::iter;

use crate::config::Config;
use crate::entity::money::{Currency, Money};
use crate::error::CliError;
use crate::CliResult;

pub fn editor() -> CliResult<String> {
    std::env::var("EDITOR").map_err(|_| CliError::UndefinedEditor)
}

pub fn random_pass() -> Option<String> {
    let mut rng = rand::thread_rng();
    let chars: String = iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .take(32)
        .collect();

    Some(chars)
}

pub fn currency(value: Option<&String>, config: &Config) -> CliResult<Currency> {
    let currency_code = value.unwrap_or(&config.currency);

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
        format!("{}", value)
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
