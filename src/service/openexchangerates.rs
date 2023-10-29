//! A library for accessing OpenExchangeRates API.

use serde::Deserialize;

use std::collections::BTreeMap;

#[derive(Deserialize, Debug)]
pub struct ExchangeRate {
    pub disclaimer: String,
    pub license: String,
    pub timestamp: i64,
    pub base: String,
    pub rates: BTreeMap<String, f32>,
}

pub struct Client {
    app_id: String,
    http: reqwest::blocking::Client,
}

impl Client {
    /// Create a new client that is ready to interact with the API.
    pub fn new(app_id: String) -> Self {
        Self {
            app_id,
            http: reqwest::blocking::Client::new(),
        }
    }

    /// Get the latest exchange rates.
    ///
    /// The corresponding endpoint in OpenExchangeRates is documented in [here](https://docs.openexchangerates.org/docs/latest-json).
    pub fn latest(self) -> anyhow::Result<ExchangeRate> {
        let url = &format!(
            "https://openexchangerates.org/api/latest.json?app_id={}",
            self.app_id
        );

        let body = self.http.get(url).send()?.text()?;

        let deserialized: ExchangeRate = serde_json::from_str(&body)?;

        Ok(deserialized)
    }
}
