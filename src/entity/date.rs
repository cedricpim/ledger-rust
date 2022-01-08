use chrono::format::strftime::StrftimeItems;
use chrono::format::DelayedFormat;
use chrono::naive::NaiveDate;
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};

use crate::error::CliError;
use crate::CliResult;

static DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Date {
    value: chrono::naive::NaiveDate,
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format(DATE_FORMAT))
    }
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
        serializer.serialize_str(&self.value.format(DATE_FORMAT).to_string())
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Date::parse(&s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl std::str::FromStr for Date {
    type Err = crate::error::CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Date::parse(s)
    }
}

impl Date {
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Date {
        chrono::naive::NaiveDate::from_ymd(year, month, day).into()
    }

    pub fn today() -> Date {
        chrono::Utc::today().naive_local().into()
    }

    pub fn pred(self) -> Date {
        self.value.pred().into()
    }

    pub fn year(self) -> i32 {
        self.value.year()
    }

    pub fn month(self) -> u32 {
        self.value.month()
    }

    pub fn since(self, rhs: Date) -> chrono::Duration {
        self.value.signed_duration_since(rhs.value)
    }

    pub fn future(self) -> bool {
        self.value > chrono::Local::now().naive_local().date()
    }

    pub fn format(self, fmt: &str) -> DelayedFormat<StrftimeItems> {
        self.value.format(fmt)
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
            val => match NaiveDate::parse_from_str(val, DATE_FORMAT) {
                Ok(value) => Ok(value.into()),
                Err(_) => match DateTime::parse_from_rfc3339(val) {
                    Ok(datetime) => Ok(datetime.naive_local().date().into()),
                    Err(_) => Err(CliError::InvalidDateFormat {
                        date: value.to_string(),
                    }),
                },
            },
        }
    }
}
