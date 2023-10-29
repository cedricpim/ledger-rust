use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Mul, Sub};

use crate::exchange::Exchange;

static DIFFERENT_CURRENCIES: &str = "Cannot perform operations between different currencies";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Currency {
    value: iso_currency::Currency,
}

impl Default for Currency {
    fn default() -> Self {
        Currency {
            value: iso_currency::Currency::EUR,
        }
    }
}

impl From<Currency> for iso_currency::Currency {
    fn from(source: Currency) -> iso_currency::Currency {
        source.value
    }
}

impl<'a> From<&'a Currency> for iso_currency::Currency {
    fn from(source: &'a Currency) -> iso_currency::Currency {
        source.value
    }
}

impl From<iso_currency::Currency> for Currency {
    fn from(value: iso_currency::Currency) -> Currency {
        Currency { value }
    }
}

impl Serialize for Currency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.value.code())
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
    pub fn parse(code: &str) -> anyhow::Result<Currency> {
        match iso_currency::Currency::from_code(code) {
            Some(value) => Ok(value.into()),
            None => Err(anyhow!(
                "The currency code '{}' does not exist",
                code.to_string()
            )),
        }
    }

    pub fn decimal_places(self) -> u16 {
        self.value.exponent().unwrap_or(0)
    }

    pub fn code(self) -> String {
        self.value.code().to_string()
    }

    pub fn symbol(self) -> String {
        self.value.symbol().symbol
    }
}

#[derive(Default, Debug, Copy, Clone, Eq)]
pub struct Money {
    value: i64,
    currency: Currency,
}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Money) -> Option<Ordering> {
        if self.currency == other.currency {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Money) -> Ordering {
        self.value.abs().cmp(&other.value.abs())
    }
}

impl PartialEq for Money {
    fn eq(&self, other: &Money) -> bool {
        self.currency == other.currency && self.value == other.value
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.to_display(), self.currency.symbol())
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        if self.currency != other.currency {
            crate::werr!(2, "{} + {}: {}", self, other, DIFFERENT_CURRENCIES)
        };

        Self {
            value: self.value + other.value,
            currency: self.currency,
        }
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        if self.currency != other.currency {
            crate::werr!(2, "{} += {}: {}", self, other, DIFFERENT_CURRENCIES)
        };

        *self = Self {
            value: self.value + other.value,
            currency: self.currency,
        }
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        if self.currency != other.currency {
            crate::werr!(2, "{} - {}: {}", self, other, DIFFERENT_CURRENCIES)
        };

        Self {
            value: self.value - other.value,
            currency: self.currency,
        }
    }
}

impl Mul<i64> for Money {
    type Output = Self;

    fn mul(self, other: i64) -> Self {
        Self {
            value: self.value * other,
            currency: self.currency,
        }
    }
}

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_storage())
    }
}

impl Money {
    pub fn parse(value: &str, currency: Currency) -> anyhow::Result<Money> {
        let val = value.parse::<f64>()?;
        let cents = val * (10_i32.pow(currency.decimal_places().into())) as f64;

        Ok(Money {
            value: cents.round() as i64,
            currency,
        })
    }

    pub fn to_display(self) -> String {
        let val = self.to_number().abs().to_string();

        let (major, minor) = match val.rfind('.') {
            None => (&val[..], ".00"),
            Some(index) => val.split_at(index),
        };
        let split_major: Vec<String> = major
            .chars()
            .rev()
            .collect::<Vec<_>>()
            .chunks(3)
            .map(|chunk| chunk.iter().rev().collect())
            .rev()
            .collect();

        let sign = if self.positive() {
            "+"
        } else if self.negative() {
            "-"
        } else {
            ""
        };

        format!(
            "{}{}{:0<width$}",
            sign,
            split_major.join(","),
            minor,
            width = 3
        )
    }

    pub fn to_storage(self) -> String {
        self.to_display().replace(',', "")
    }

    pub fn new(currency: Currency, value: i64) -> Money {
        Self { value, currency }
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn cents(&self) -> i64 {
        self.value
    }

    pub fn exchange(&self, to: Currency, exchange: &Exchange) -> anyhow::Result<Money> {
        if self.currency == to {
            return Ok(*self);
        }

        let rate = exchange.rate(self.currency(), to)? as f64;

        let dec_adjust =
            10f64.powi(to.decimal_places() as i32 - self.currency.decimal_places() as i32);
        let amount = (self.value as f64 * rate * dec_adjust).round() as i64;

        Ok(Self {
            value: amount,
            currency: to,
        })
    }

    pub fn abs(&self) -> Money {
        Money {
            value: self.value.abs(),
            currency: self.currency,
        }
    }

    pub fn zero(&self) -> bool {
        self.value == 0
    }

    pub fn positive(&self) -> bool {
        self.value > 0
    }

    pub fn negative(&self) -> bool {
        self.value < 0
    }

    pub fn to_number(self) -> f64 {
        self.value as f64 / (10_i32.pow(self.currency.decimal_places().into())) as f64
    }
}
