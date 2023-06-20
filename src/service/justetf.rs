use crate::entity::money::{Currency, Money};
use custom_error::custom_error;
use serde_json::Value;

const URL: &str =
    "https://www.justetf.com/api/etfs/cards?locale=en&currency={CURRENCY}&isin={ISIN}";

custom_error! { pub Error
    Reqwest { source: reqwest::Error }    = @{ source },
    SerdeJson { source: serde_json::Error } = @{ source },
    RepeatableReqwest { message: String } = "Repeatable error: {message}",

    Parser        = "The element provided could not be parsed",
    NameNotFound  = "Asset name could not be found",
    ValueNotFound = "Asset value could not be found",
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub isin: String,
    pub name: String,
    pub quote: Money,
}

impl Asset {
    pub fn download(isin: &str, currency: &Currency) -> Result<Asset, Error> {
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

    fn money(data: &Value) -> Result<String, Error> {
        data.pointer("/etfs/0/quote/raw")
            .map(|val| val.to_string())
            .ok_or(Error::ValueNotFound)
    }

    fn name(data: &Value) -> Result<&str, Error> {
        data.pointer("/etfs/0/name")
            .and_then(|val| val.as_str())
            .ok_or(Error::NameNotFound)
    }

    fn data(isin: &str, currency: &Currency) -> Result<Value, Error> {
        let client = reqwest::blocking::Client::new();

        let url = URL
            .replace("{ISIN}", isin)
            .replace("{CURRENCY}", &currency.code());

        let body = client.get(url).send()?.text()?;

        let response: Value = serde_json::from_str(&body)?;

        Ok(response)
    }
}
