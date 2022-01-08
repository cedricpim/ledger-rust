use custom_error::custom_error;
use kuchiki::traits::*;
use retry::{retry_with_index, OperationResult};

const URL: &str = "https://www.justetf.com/en/etf-profile.html";
const NAME_CSS: &str = ".h1";
const VALUE_CSS: &str = "div.val span";
const MAXIMUM_RETRIES: u64 = 5;

custom_error! { pub Error
    Reqwest { source: reqwest::Error }    = @{ source },
    RepeatableReqwest { message: String } = "Repeatable error: {message}",

    Parser        = "The element provided could not be parsed",
    NameNotFound  = "Asset name could not be found",
    ValueNotFound = "Asset value could not be found",
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub isin: String,
    pub name: String,
    pub value: String,
    pub currency: String,
}

impl Asset {
    pub fn download(isin: &str) -> Result<Asset, Error> {
        let document = Asset::document(isin)?;

        let name = Asset::name(&document)?;

        let (currency, value) = Asset::money(&document)?;

        Ok(Asset {
            isin: isin.to_string(),
            name,
            value,
            currency,
        })
    }

    fn money(document: &kuchiki::NodeRef) -> Result<(String, String), Error> {
        match document.select(VALUE_CSS) {
            Ok(val) => {
                let values: Vec<String> = val.take(2).map(|v| v.text_contents()).collect();

                if values.len() == 2 {
                    Ok((values[0].to_string(), values[1].to_string()))
                } else {
                    Err(Error::ValueNotFound)
                }
            }
            Err(_) => Err(Error::ValueNotFound),
        }
    }

    fn name(document: &kuchiki::NodeRef) -> Result<String, Error> {
        match document.select_first(NAME_CSS) {
            Ok(val) => Ok(val.text_contents()),
            Err(_) => Err(Error::NameNotFound),
        }
    }

    fn document(isin: &str) -> Result<kuchiki::NodeRef, Error> {
        let client = reqwest::blocking::Client::new();

        let body = retry_with_index(retry::delay::Fixed::from_millis(1000), |current_try| {
            let response = client.get(URL).query(&[("isin", isin)]).send().unwrap();
            match response.text() {
                Ok(val) => OperationResult::Ok(val),
                Err(err) => {
                    println!("[{}] Retrying...", isin);
                    if current_try > MAXIMUM_RETRIES {
                        OperationResult::Err(err)
                    } else {
                        OperationResult::Retry(err)
                    }
                }
            }
        })
        .map_err(|err| Error::RepeatableReqwest {
            message: format!("{:?}", err),
        })?;

        Ok(kuchiki::parse_html().one(body))
    }
}
