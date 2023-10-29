use anyhow::anyhow;
use serde_json::Value;

use crate::entity::money::{Currency, Money};

const URL: &str =
    "https://www.justetf.com/api/etfs/cards?locale=en&currency={CURRENCY}&isin={ISIN}";

#[derive(Debug, Clone)]
pub struct Asset {
    pub isin: String,
    pub name: String,
    pub quote: Money,
}

impl Asset {
    pub fn download(isin: &str, currency: &Currency) -> anyhow::Result<Asset> {
        let data = Asset::data(isin, currency)?;

        let name = Asset::name(&data)?;

        let value = Asset::money(&data)?;

        let quote = Money::parse(&value, *currency).unwrap_or_else(|e| crate::werr!(1, "{}", e));

        Ok(Asset {
            isin: isin.to_string(),
            name: name.to_string(),
            quote,
        })
    }

    fn money(data: &Value) -> anyhow::Result<String> {
        data.pointer("/etfs/0/quote/raw")
            .map(|val| val.to_string())
            .ok_or(anyhow!("Asset value could not be found"))
    }

    fn name(data: &Value) -> anyhow::Result<&str> {
        data.pointer("/etfs/0/name")
            .and_then(|val| val.as_str())
            .ok_or(anyhow!("Asset name could not be found"))
    }

    fn data(isin: &str, currency: &Currency) -> anyhow::Result<Value> {
        let client = reqwest::blocking::Client::new();

        let url = URL
            .replace("{ISIN}", isin)
            .replace("{CURRENCY}", &currency.code());

        let body = client.get(url).send()?.text()?;

        let response: Value = serde_json::from_str(&body)?;

        Ok(response)
    }
}
