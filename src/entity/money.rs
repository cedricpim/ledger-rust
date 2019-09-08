use serde::{Deserialize, Serialize};
use steel_cent::formatting::{FormatPart, FormatSpec};

use std::ops::Add;
use std::collections::HashMap;

use crate::error::CliError;
use crate::exchange::Exchange;
use crate::CliResult;

lazy_static!{
    static ref SYMBOLS: HashMap<&'static str, &'static str> = [
        ("EUR", "€"),
        ("PLN", "zł"),
        ("USD", "$"),
    ].iter().copied().collect();
}

#[derive(Debug, Copy, Clone)]
pub struct Currency {
    value: steel_cent::currency::Currency,
}

impl Default for Currency {
    fn default() -> Self {
        Currency {
            value: steel_cent::currency::EUR,
        }
    }
}

impl From<Currency> for steel_cent::currency::Currency {
    fn from(source: Currency) -> steel_cent::currency::Currency {
        source.value
    }
}

impl<'a> From<&'a Currency> for steel_cent::currency::Currency {
    fn from(source: &'a Currency) -> steel_cent::currency::Currency {
        source.value
    }
}

impl From<steel_cent::currency::Currency> for Currency {
    fn from(value: steel_cent::currency::Currency) -> Currency {
        Currency { value }
    }
}

impl Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value.code())
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Currency::parse(&s)
            .map_err(|_| serde::de::Error::custom(format!("No matching currency for code: {}", s)))
    }
}

impl Currency {
    pub fn parse(code: &str) -> CliResult<Currency> {
        match steel_cent::currency::with_code(code) {
            Some(value) => Ok(value.into()),
            None => Err(CliError::IncorrectCurrencyCode {
                code: code.to_string(),
            }),
        }
    }

    pub fn decimal_places(self) -> u8 {
        self.value.decimal_places()
    }

    pub fn code(self) -> String {
        self.value.code()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Money {
    value: steel_cent::Money,
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code: &str = &self.currency().code();

        write!(f, "{}{}", self.to_string(), SYMBOLS.get(code).unwrap_or(&code))
    }
}

impl From<Money> for steel_cent::Money {
    fn from(source: Money) -> steel_cent::Money {
        source.value
    }
}

impl<'a> From<&'a Money> for steel_cent::Money {
    fn from(source: &'a Money) -> steel_cent::Money {
        source.value
    }
}

impl From<steel_cent::Money> for Money {
    fn from(value: steel_cent::Money) -> Money {
        Money { value }
    }
}

impl Default for Money {
    fn default() -> Self {
        Money {
            value: steel_cent::Money::of_minor(Currency::default().into(), 0),
        }
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self { value: self.value + other.value }
    }
}

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", &self.to_string().replace(",", "")))
    }
}

impl Money {
    pub fn parse(value: &str, currency: Currency) -> CliResult<Money> {
        let parser = FormatSpec::new(
            '\0',
            '.',
            vec![
                FormatPart::OptionalMinus,
                FormatPart::Amount,
                FormatPart::CurrencySymbol,
            ],
        )
        .parser();

        match parser.parse::<steel_cent::Money>(&Money::formatted_value(value, currency)) {
            Err(err) => Err(CliError::from(err)),
            Ok(val) => Ok(val.into()),
        }
    }

    pub fn to_string(&self) -> String {
        let val = FormatSpec::new(',', '.', vec![FormatPart::Amount])
            .display_for(&self.value)
            .to_string();

        let (integer, fractional) = match val.rfind('.') {
            None => (&val[..], ".00"),
            Some(index) => val.split_at(index),
        };

        let sign = if self.value == self.value.abs() {
            "+"
        } else {
            "-"
        };

        format!("{}{}{:0<width$}", sign, integer, fractional, width = 3)
    }

    pub fn new(currency: Currency, value: i64) -> Money {
        steel_cent::Money::of_minor(currency.into(), value).into()
    }

    pub fn currency(&self) -> Currency {
        self.value.currency.into()
    }

    pub fn exchange(&self, to: Option<Currency>, exchange: &Exchange) -> CliResult<Money> {
        match to {
            Some(currency) => {
                let rate = exchange.rate(self.currency(), currency)?;
                let exchanged = self.value.convert_to(currency.into(), rate.into());
                Ok(exchanged.into())
            }
            None => Ok(self.to_owned()),
        }
    }

    pub fn zero(&self) -> bool {
        self.value.minor_amount() == 0
    }

    pub fn positive(&self) -> bool {
        self.value.minor_amount() > 0
    }

    pub fn negative(&self) -> bool {
        self.value.minor_amount() < 0
    }

    fn formatted_value(value: &str, currency: Currency) -> String {
        let (integer, fractional) = match value.rfind('.') {
            None => (value, "."),
            Some(index) => value.split_at(index),
        };

        let width = currency.decimal_places().into();

        if fractional.len() > width {
            format!(
                "{}{:.*}{}",
                integer.replace("+", ""),
                width + 1,
                fractional,
                currency.code()
            )
        } else {
            format!(
                "{}{:0<width$}{}",
                integer.replace("+", ""),
                fractional,
                currency.code(),
                width = width + 1
            )
        }
    }
}
