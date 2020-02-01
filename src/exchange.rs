use serde::{Deserialize, Serialize};
use serde_yaml;

use std::collections::BTreeMap;
use std::io::Write;

use crate::service::openexchangerates;
use crate::config::Config;
use crate::entity::money::Currency;
use crate::error::CliError;
use crate::CliResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exchange {
    timestamp: i64,
    base: String,
    rates: BTreeMap<String, f32>,
}

impl From<openexchangerates::ExchangeRate> for Exchange {
    fn from(item: openexchangerates::ExchangeRate) -> Self {
        Exchange {
            timestamp: item.timestamp,
            base: item.base,
            rates: item.rates,
        }
    }
}

impl Exchange {
    pub fn new(config: &Config) -> CliResult<Exchange> {
        if config.exchange().cached() {
            Exchange::load(&config)
        } else {
            Exchange::download(&config)
        }
    }

    pub fn rate(&self, from: Currency, to: Currency) -> CliResult<f32> {
        match self.rates.get(&to.code()) {
            None => Err(CliError::MissingExchangeRate {
                code: to.code(),
            }),
            Some(dividend) => match self.rates.get(&from.code()) {
                None => Err(CliError::MissingExchangeRate {
                    code: from.code(),
                }),
                Some(divisor) => Ok(dividend / divisor),
            },
        }
    }

    fn load(config: &Config) -> CliResult<Exchange> {
        serde_yaml::from_reader(config.exchange().open()?).map_err(CliError::from)
    }

    fn download(config: &Config) -> CliResult<Exchange> {
        match openexchangerates::Client::new(config.exchange().key()).latest() {
            Ok(result) => Exchange::store(result.into(), &config),
            Err(openexchangerates::Error::Reqwest { .. }) => match Exchange::load(&config) {
                Ok(val) => Ok(val),
                Err(_) => Err(CliError::ExchangeInternetRequired),
            },
            Err(err) => Err(CliError::from(err)),
        }
    }

    fn store(exchange: Exchange, config: &Config) -> CliResult<Exchange> {
        let mut file = config.exchange().create()?;
        let yaml = serde_yaml::to_string(&exchange)?;
        file.write_all(yaml.as_bytes())?;
        Ok(exchange)
    }
}
