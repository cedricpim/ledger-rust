use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

use std::fs::File;

use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::{Currency, Money};
use crate::error::CliError;
use crate::exchange::Exchange;
use crate::CliResult;

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Transaction {
    pub account: String,
    pub date: Date,
    pub category: String,
    pub description: String,
    pub quantity: String,
    pub venue: String,
    pub amount: Money,
    pub currency: Currency,
    pub trip: String,
}

impl Transaction {
    pub fn build(values: Vec<String>) -> CliResult<Transaction> {
        let currency = Currency::parse(&values[7])?;

        Ok(Transaction {
            account: values[0].to_string(),
            date: Date::parse(&values[1])?,
            category: values[2].to_string(),
            description: values[3].to_string(),
            quantity: values[4].to_string(),
            venue: values[5].to_string(),
            amount: Money::parse(&values[6], currency)?,
            currency,
            trip: values[8].to_string(),
        })
    }
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

    fn account(&self) -> String {
        self.account.to_owned()
    }

    fn category(&self) -> String {
        self.category.to_owned()
    }

    fn date(&self) -> Date {
        self.date
    }

    fn amount(&self) -> Money {
        self.amount
    }

    fn currency(&self) -> Currency {
        self.currency
    }

    fn write(&self, wrt: &mut csv::Writer<File>) -> CliResult<()> {
        wrt.serialize(self).map_err(CliError::from)
    }

    fn exchange(&self, to: Option<Currency>, exchange: &Exchange) -> CliResult<Line> {
        let money = self.amount.exchange(to, &exchange)?;

        Ok(Transaction {
            account: self.account.clone(),
            date: self.date,
            category: self.category.clone(),
            description: self.description.clone(),
            quantity: self.quantity.clone(),
            venue: self.venue.clone(),
            currency: money.currency(),
            amount: money,
            trip: self.trip.clone(),
        }
        .into())
    }
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "PascalCase")]
        enum Field {
            Account,
            Date,
            Category,
            Description,
            Quantity,
            Venue,
            Amount,
            Currency,
            Trip,
        }

        struct TransactionVisitor;

        impl<'de> Visitor<'de> for TransactionVisitor {
            type Value = Transaction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Transaction")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Transaction, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut account = None;
                let mut date = None;
                let mut category = None;
                let mut description = None;
                let mut quantity = None;
                let mut venue = None;
                let mut amount = None;
                let mut currency = None;
                let mut trip = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Account => {
                            if account.is_some() {
                                return Err(de::Error::duplicate_field("account"));
                            }
                            account = Some(map.next_value()?);
                        }
                        Field::Date => {
                            if date.is_some() {
                                return Err(de::Error::duplicate_field("date"));
                            }
                            date = Some(map.next_value()?);
                        }
                        Field::Category => {
                            if category.is_some() {
                                return Err(de::Error::duplicate_field("category"));
                            }
                            category = Some(map.next_value()?);
                        }
                        Field::Description => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }
                            description = Some(map.next_value()?);
                        }
                        Field::Quantity => {
                            if quantity.is_some() {
                                return Err(de::Error::duplicate_field("quantity"));
                            }
                            quantity = Some(map.next_value()?);
                        }
                        Field::Venue => {
                            if venue.is_some() {
                                return Err(de::Error::duplicate_field("venue"));
                            }
                            venue = Some(map.next_value()?);
                        }
                        Field::Amount => {
                            if amount.is_some() {
                                return Err(de::Error::duplicate_field("amount"));
                            }
                            amount = Some(map.next_value()?);
                        }
                        Field::Currency => {
                            if currency.is_some() {
                                return Err(de::Error::duplicate_field("currency"));
                            }
                            currency = Some(map.next_value()?);
                        }
                        Field::Trip => {
                            if trip.is_some() {
                                return Err(de::Error::duplicate_field("trip"));
                            }
                            trip = Some(map.next_value()?);
                        }
                    }
                }

                let amount = amount.ok_or_else(|| de::Error::missing_field("amount"))?;
                let currency = currency.ok_or_else(|| de::Error::missing_field("currency"))?;

                Ok(Transaction {
                    account: account.ok_or_else(|| de::Error::missing_field("account"))?,
                    date: date.ok_or_else(|| de::Error::missing_field("date"))?,
                    category: category.ok_or_else(|| de::Error::missing_field("category"))?,
                    description: description
                        .ok_or_else(|| de::Error::missing_field("description"))?,
                    quantity: quantity.ok_or_else(|| de::Error::missing_field("quantity"))?,
                    venue: venue.ok_or_else(|| de::Error::missing_field("venue"))?,
                    amount: Money::parse(amount, currency).map_err(de::Error::custom)?,
                    currency,
                    trip: trip.ok_or_else(|| de::Error::missing_field("trip"))?,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "account",
            "date",
            "category",
            "description",
            "quantity",
            "venue",
            "amount",
            "currency",
            "trip",
        ];
        deserializer.deserialize_struct("Transaction", FIELDS, TransactionVisitor)
    }
}