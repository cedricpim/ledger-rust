use serde::{Deserialize, Serialize};
use steel_cent::formatting::{FormatPart, FormatSpec};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::ops::{Add, AddAssign, Mul, Sub};

use crate::error::CliError;
use crate::exchange::Exchange;
use crate::CliResult;

lazy_static! {
    // Commonly used currencies
    static ref SYMBOLS: HashMap<&'static str, &'static str> = {
        [
            ("ars", "$"),
            ("brl", "R$"),
            ("chf", "CHF"),
            ("cny", "¥"),
            ("eur", "€"),
            ("jpy", "¥"),
            ("pln", "zł"),
            ("usd", "$"),
            ("vnd", "₫"),
        ]
        .iter()
        .copied()
        .collect()
    };
}

lazy_static! {
    static ref CURRENCIES: HashMap<String, CurrencyInfo> = {
        match File::open("data/currencies.json") {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).unwrap_or_default()
            }
            Err(_) => HashMap::new(),
        }
    };
}

#[derive(Deserialize, Debug)]
pub struct CurrencyInfo {
    #[serde(skip)]
    priority: i64,
    iso_code: String,
    name: String,
    symbol: String,
    #[serde(skip)]
    alternate_symbols: Vec<String>,
    #[serde(skip)]
    subunit: String,
    #[serde(skip)]
    subunit_to_unit: i64,
    #[serde(skip)]
    symbol_first: bool,
    #[serde(skip)]
    html_entity: String,
    #[serde(skip)]
    decimal_mark: String,
    #[serde(skip)]
    thousands_separator: String,
    #[serde(skip)]
    iso_numeric: String,
    #[serde(skip)]
    smallest_denomination: i64,
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

#[derive(Debug, Copy, Clone, Eq)]
pub struct Money {
    value: steel_cent::Money,
}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Money) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Money) -> Ordering {
        self.cents().abs().cmp(&other.cents().abs())
    }
}

impl PartialEq for Money {
    fn eq(&self, other: &Money) -> bool {
        self.value == other.value
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.to_storage(), self.symbol())
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
        Self {
            value: self.value + other.value,
        }
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            value: self.value + other.value,
        }
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            value: self.value - other.value,
        }
    }
}

impl Mul<u64> for Money {
    type Output = Self;

    fn mul(self, other: u64) -> Self {
        let cents = self.value.minor_amount();

        Self {
            value: steel_cent::Money::of_minor(self.value.currency, cents * other as i64),
        }
    }
}

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_storage().replace(",", ""))
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

    pub fn to_storage(&self) -> String {
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

    pub fn exchange(&self, to: Currency, exchange: &Exchange) -> CliResult<Money> {
        let rate = exchange.rate(self.currency(), to)?;
        Ok(self.value.convert_to(to.into(), rate.into()).into())
    }

    pub fn abs(&self) -> Money {
        self.value.abs().into()
    }

    pub fn cents(&self) -> i64 {
        self.value.minor_amount()
    }

    pub fn zero(&self) -> bool {
        self.cents() == 0
    }

    pub fn positive(&self) -> bool {
        self.cents() > 0
    }

    pub fn negative(&self) -> bool {
        self.cents() < 0
    }

    pub fn to_number(&self) -> f64 {
        self.cents() as f64 / (10_i32.pow(self.currency().decimal_places().into())) as f64
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

    fn symbol(&self) -> String {
        let code = self.currency().code().to_lowercase();

        match SYMBOLS.get(code.as_str()) {
            Some(val) => (*val).to_string(),
            None => CURRENCIES
                .get(code.as_str())
                .map_or(code, |v| v.symbol.to_string()),
        }
    }
}
