use openexchangerates;
use serde::{Deserialize, Serialize};
use serde_yaml;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::config::Config;
use crate::entity::money::Currency;
use crate::error::CliError;
use crate::CliResult;

#[derive(Debug, Serialize, Deserialize)]
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
        if Exchange::valid_cache(&config) {
            Exchange::load(&config)
        } else {
            Exchange::download(&config)
        }
    }

    pub fn rate(&self, from: Currency, to: Currency) -> CliResult<f32> {
        match self.rates.get(&to.code()) {
            None => Err(CliError::MissingExchangeRate {
                code: to.code().to_string(),
            }),
            Some(dividend) => match self.rates.get(&from.code()) {
                None => Err(CliError::MissingExchangeRate {
                    code: from.code().to_string(),
                }),
                Some(divisor) => Ok(dividend / divisor),
            },
        }
    }

    fn valid_cache(config: &Config) -> bool {
        let path = Path::new(&config.exchange.cache_file);

        let mtime = path
            .metadata()
            .and_then(|v| v.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let default = Duration::new(config.exchange.ttl, 0);

        path.exists()
            && SystemTime::now()
                .duration_since(mtime)
                .unwrap_or(default)
                .as_secs()
                < config.exchange.ttl
    }

    fn load(config: &Config) -> CliResult<Exchange> {
        let file = File::open(&config.exchange.cache_file)?;
        serde_yaml::from_reader(file).map_err(CliError::from)
    }

    fn download(config: &Config) -> CliResult<Exchange> {
        match openexchangerates::Client::new(&config.exchange.api_key).latest() {
            Ok(result) => Exchange::store(result.into(), &config.exchange.cache_file),
            Err(openexchangerates::error::Error::Hyper(_)) => match Exchange::load(&config) {
                Ok(val) => Ok(val),
                Err(_) => Err(CliError::ExchangeInternetRequired),
            },
            Err(err) => Err(CliError::from(err)),
        }
    }

    fn store(exchange: Exchange, location: &str) -> CliResult<Exchange> {
        let mut file = File::create(location)?;
        let yaml = serde_yaml::to_string(&exchange)?;
        file.write_all(yaml.as_bytes())?;
        Ok(exchange)
    }
}
