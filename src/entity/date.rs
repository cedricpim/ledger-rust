use chrono::naive::NaiveDate;
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};

use crate::error::CliError;
use crate::CliResult;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Date {
    value: chrono::naive::NaiveDate,
}

impl Default for Date {
    fn default() -> Self {
        Date {
            value: Utc::today().naive_local(),
        }
    }
}

impl From<Date> for chrono::naive::NaiveDate {
    fn from(source: Date) -> chrono::naive::NaiveDate {
        source.value
    }
}

impl From<chrono::naive::NaiveDate> for Date {
    fn from(value: chrono::naive::NaiveDate) -> Date {
        Date { value }
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value.format("%Y-%m-%d").to_string())
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Date::parse(&s).map_err(|_| {
            serde::de::Error::custom(format!(
                "Invalid format for date: {} (only accept %Y-%m-%d)",
                s
            ))
        })
    }
}

impl Date {
    pub fn today() -> Date {
        chrono::Utc::today().naive_local().into()
    }

    pub fn year(self) -> i32 {
        self.value.year()
    }

    pub fn month(self) -> u32 {
        self.value.month()
    }

    pub fn end_of_month(self) -> Date {
        match self.month() {
            month @ 12 => chrono::naive::NaiveDate::from_ymd(self.year(), month, 31),
            month => chrono::naive::NaiveDate::from_ymd(self.year(), month + 1, 1).pred(),
        }
        .into()
    }

    pub fn parse(value: &str) -> CliResult<Date> {
        match value {
            "" => Ok(Default::default()),
            val => match NaiveDate::parse_from_str(val, "%Y-%m-%d") {
                Ok(value) => Ok(value.into()),
                Err(err) => Err(CliError::from(err)),
            },
        }
    }
}
