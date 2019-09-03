use chrono::naive::NaiveDate;
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
pub struct Entry {
    pub date: Date,
    pub invested: Money,
    pub investment: Money,
    pub amount: Money,
    pub currency: Currency,
}

impl Entry {
    pub fn build(values: Vec<String>) -> CliResult<Entry> {
        let currency = Currency::parse(&values[4])?;

        Ok(Entry {
            date: Date::parse(&values[0])?,
            invested: Money::parse(&values[1], &currency)?,
            investment: Money::parse(&values[2], &currency)?,
            amount: Money::parse(&values[3], &currency)?,
            currency,
        })
    }
}

impl Liner for Entry {
    fn headers(&self) -> Vec<&'static str> {
        vec!["Date", "Invested", "Investment", "Amount", "Currency"]
    }

    fn category(&self) -> String {
        "".to_string()
    }

    fn date(&self) -> NaiveDate {
        self.date.clone().into()
    }

    fn write(&self, wrt: &mut csv::Writer<File>) -> CliResult<()> {
        wrt.serialize(self).map_err(CliError::from)
    }

    fn exchange(&self, to: &Option<Currency>, exchange: &Exchange) -> CliResult<Line> {
        let money = self.amount.exchange(&to, &exchange)?;

        Ok(Entry {
            date: self.date.clone(),
            invested: self.invested.exchange(&to, &exchange)?,
            investment: self.investment.exchange(&to, &exchange)?,
            currency: money.currency(),
            amount: money,
        }
        .into())
    }
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "PascalCase")]
        enum Field {
            Date,
            Invested,
            Investment,
            Amount,
            Currency,
        }

        struct EntryVisitor;

        impl<'de> Visitor<'de> for EntryVisitor {
            type Value = Entry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Entry")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Entry, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut date = None;
                let mut invested = None;
                let mut investment = None;
                let mut amount = None;
                let mut currency = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Date => {
                            if date.is_some() {
                                return Err(de::Error::duplicate_field("date"));
                            }
                            date = Some(map.next_value()?);
                        }
                        Field::Invested => {
                            if invested.is_some() {
                                return Err(de::Error::duplicate_field("invested"));
                            }
                            invested = Some(map.next_value()?);
                        }
                        Field::Investment => {
                            if investment.is_some() {
                                return Err(de::Error::duplicate_field("investment"));
                            }
                            investment = Some(map.next_value()?);
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
                    }
                }

                let invested = invested.ok_or_else(|| de::Error::missing_field("invested"))?;
                let investment =
                    investment.ok_or_else(|| de::Error::missing_field("investment"))?;
                let amount = amount.ok_or_else(|| de::Error::missing_field("amount"))?;
                let currency = currency.ok_or_else(|| de::Error::missing_field("currency"))?;

                Ok(Entry {
                    date: date.ok_or_else(|| de::Error::missing_field("date"))?,
                    invested: Money::parse(invested, &currency).map_err(de::Error::custom)?,
                    investment: Money::parse(investment, &currency).map_err(de::Error::custom)?,
                    amount: Money::parse(amount, &currency).map_err(de::Error::custom)?,
                    currency,
                })
            }
        }

        const FIELDS: &[&str] =
            &["date", "invested", "investment", "amount", "currency"];
        deserializer.deserialize_struct("Entry", FIELDS, EntryVisitor)
    }
}
