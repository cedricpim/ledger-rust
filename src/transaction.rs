use chrono::naive::NaiveDate;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::error::CliError;
use crate::CliResult;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct Transaction {
    account: String,
    date: NaiveDate,
    category: String,
    description: String,
    quantity: Decimal,
    venue: String,
    amount: Decimal,
    currency: String,
    trip: String,
}

impl Transaction {
    pub fn headers() -> Vec<&'static str> {
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

    pub fn build(values: Vec<String>) -> CliResult<Transaction> {
        Ok(Transaction {
            account: values[0].to_string(),
            date: Transaction::parse_date(&values[1])?,
            category: values[2].to_string(),
            description: values[3].to_string(),
            quantity: Decimal::from_str(&values[4]).map_err(CliError::from)?,
            venue: values[5].to_string(),
            amount: Decimal::from_str(&values[6]).map_err(CliError::from)?,
            currency: values[7].to_string(),
            trip: values[8].to_string(),
        })
    }

    fn parse_date(value: &str) -> CliResult<NaiveDate> {
        let parse = NaiveDate::parse_from_str;

        match value {
            "" => Ok(Utc::today().naive_local()),
            val => parse(val, "%Y-%m-%d")
                .or_else(|_| parse(val, "%Y/%m/%d"))
                .or_else(|_| parse(val, "%d-%m-%Y"))
                .or_else(|_| parse(val, "%d/%m/%Y"))
                .map_err(CliError::from),
        }
    }
}
