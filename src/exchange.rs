use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::config::Config;
use crate::entity::money::Currency;
use crate::error::CliError;
use crate::service::openexchangerates;
use crate::xdg::Xdg;
use crate::CliResult;

const EXCHANGE_CACHE_FILENAME: &str = "exchange.yml";
const EXCHANGE_CACHE_TTL: u64 = 43200; // 12 hours

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exchange {
    timestamp: i64,
    base: String,
    rates: BTreeMap<String, f32>,
}

#[derive(Debug)]
pub struct Cache {
    filepath: String,
}

impl Cache {
    pub fn new() -> CliResult<Self> {
        Ok(Self {
            filepath: Xdg::Cache(EXCHANGE_CACHE_FILENAME.to_string()).filepath()?,
        })
    }

    pub fn valid(&self) -> bool {
        let path = Path::new(&self.filepath);

        let mtime = path
            .metadata()
            .and_then(|v| v.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let default = Duration::new(EXCHANGE_CACHE_TTL, 0);

        let interval = SystemTime::now()
            .duration_since(mtime)
            .unwrap_or(default)
            .as_secs();

        path.exists() && interval < EXCHANGE_CACHE_TTL
    }

    pub fn open(&self) -> CliResult<File> {
        File::open(&self.filepath).map_err(CliError::from)
    }

    pub fn create(&self) -> CliResult<File> {
        File::create(&self.filepath).map_err(CliError::from)
    }
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
        let cache = Cache::new()?;

        if cache.valid() {
            Exchange::load(&cache)
        } else {
            Exchange::download(&config, &cache)
        }
    }

    pub fn rate(&self, from: Currency, to: Currency) -> CliResult<f32> {
        match self.rates.get(&to.code()) {
            None => Err(CliError::MissingExchangeRate { code: to.code() }),
            Some(dividend) => match self.rates.get(&from.code()) {
                None => Err(CliError::MissingExchangeRate { code: from.code() }),
                Some(divisor) => Ok(dividend / divisor),
            },
        }
    }

    fn load(cache: &Cache) -> CliResult<Exchange> {
        serde_yaml::from_reader(cache.open()?).map_err(CliError::from)
    }

    fn download(config: &Config, cache: &Cache) -> CliResult<Exchange> {
        match openexchangerates::Client::new(config.exchange_key()).latest() {
            Ok(result) => Exchange::store(result.into(), &cache),
            Err(openexchangerates::Error::Reqwest { .. }) => match Exchange::load(&cache) {
                Ok(val) => Ok(val),
                Err(_) => Err(CliError::ExchangeInternetRequired),
            },
            Err(err) => Err(CliError::from(err)),
        }
    }

    fn store(exchange: Exchange, cache: &Cache) -> CliResult<Exchange> {
        let mut file = cache.create()?;
        let yaml = serde_yaml::to_string(&exchange)?;
        file.write_all(yaml.as_bytes())?;
        Ok(exchange)
    }
}
