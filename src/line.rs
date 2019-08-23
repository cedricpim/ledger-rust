use chrono::naive::NaiveDate;
use enum_dispatch::enum_dispatch;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use std::str::FromStr;

use crate::{error, util, CliResult};

#[enum_dispatch]
pub enum Line {
    Transaction,
    Entry,
}

impl Line {
    pub fn default(networth: bool) -> Self {
        match networth {
            false => Transaction::default().into(),
            true => Entry::default().into(),
        }
    }
}

#[enum_dispatch(Line)]
pub trait Liner {
    fn headers(&self) -> Vec<&'static str>;
    fn category(&self) -> String;
    fn date(&self) -> NaiveDate;
}

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[serde(rename_all = "PascalCase")]
#[derivative(Default)]
pub struct Transaction {
    account: String,
    #[derivative(Default(value = "util::default_date()"))]
    date: NaiveDate,
    category: String,
    description: String,
    quantity: Decimal,
    venue: String,
    amount: Decimal,
    currency: String,
    trip: String,
}

impl Liner for Transaction {
    fn headers(&self) -> Vec<&'static str> {
        vec![
            "Account",
            "Date",
            "Category",
            "Description",
            "Quantity",
            "Venue",
            "Amount",
            "Currency",
            "Trip",
        ]
    }

    fn category(&self) -> String {
        self.category.to_owned()
    }

    fn date(&self) -> NaiveDate {
        self.date
    }
}

impl Transaction {
    pub fn build(values: Vec<String>) -> CliResult<Transaction> {
        Ok(Transaction {
            account: values[0].to_string(),
            date: util::parse_date(&values[1])?,
            category: values[2].to_string(),
            description: values[3].to_string(),
            quantity: Decimal::from_str(&values[4]).map_err(error::CliError::from)?,
            venue: values[5].to_string(),
            amount: Decimal::from_str(&values[6]).map_err(error::CliError::from)?,
            currency: values[7].to_string(),
            trip: values[8].to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[serde(rename_all = "PascalCase")]
#[derivative(Default)]
pub struct Entry {
    #[derivative(Default(value = "util::default_date()"))]
    date: NaiveDate,
    invested: Decimal,
    investment: Decimal,
    amount: Decimal,
    currency: String,
}

impl Liner for Entry {
    fn headers(&self) -> Vec<&'static str> {
        vec!["Date", "Invested", "Investment", "Amount", "Currency"]
    }

    fn category(&self) -> String {
        "".to_string()
    }

    fn date(&self) -> NaiveDate {
        self.date
    }
}
