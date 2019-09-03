use serde::{Deserialize, Serialize};
use steel_cent::formatting::FormatPart::*;
use steel_cent::formatting::FormatSpec;
// use steel_cent::formatting::FormattableMoney;

use crate::error::CliError;
use crate::exchange::Exchange;
use crate::CliResult;

#[derive(Debug, Clone)]
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

    pub fn code(&self) -> String {
        self.value.code()
    }
}

#[derive(Debug, Clone)]
pub struct Money {
    value: steel_cent::Money,
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

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let custom_spec = FormatSpec::new(',', '.', vec![OptionalMinus, Amount]);
        serializer.serialize_str(&format!("{}", custom_spec.display_for(&self.value)))
    }
}

impl Money {
    pub fn parse(value: &str, currency: &Currency) -> CliResult<Money> {
        match value.parse::<f64>() {
            Err(err) => Err(CliError::from(err)),
            Ok(val) => {
                let currency: steel_cent::currency::Currency = currency.into();
                let cents = 10f64.powf(currency.decimal_places().into());

                let minor = (val * cents).round() as i64;
                Ok(steel_cent::Money::of_minor(currency, minor).into())
            }
        }
    }

    pub fn currency(&self) -> Currency {
        self.value.currency.into()
    }

    pub fn exchange(&self, to: &Option<Currency>, exchange: &Exchange) -> CliResult<Money> {
        match to {
            Some(currency) => {
                let rate = exchange.rate(&self.currency(), &currency)?;
                let exchanged = self.value.convert_to(currency.into(), rate.into());
                Ok(exchanged.into())
            }
            None => Ok(self.to_owned()),
        }
    }
}
