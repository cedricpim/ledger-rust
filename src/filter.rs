use chrono::naive::NaiveDate;
use chrono::{Datelike, Utc};

use std::ops::RangeInclusive;

use crate::line::{Line, Liner};

#[derive(Debug)]
pub struct Filter {
    year: Option<i32>,
    month: Option<u32>,
    from: Option<NaiveDate>,
    till: Option<NaiveDate>,
    categories: Vec<String>,
}

impl Filter {
    pub fn new(
        year: Option<i32>,
        month: Option<u32>,
        from: Option<NaiveDate>,
        till: Option<NaiveDate>,
        categories: Vec<String>,
    ) -> Self {
        Filter {
            year,
            month,
            from,
            till,
            categories,
        }
    }

    pub fn apply(&self, line: &Line) -> bool {
        !self.categories.contains(&line.category()) && self.period().contains(&line.date())
    }

    fn period(&self) -> RangeInclusive<NaiveDate> {
        if self.from.is_some() || self.till.is_some() {
            let start = self.from.unwrap_or(chrono::naive::MIN_DATE);
            let end = self.till.unwrap_or(chrono::naive::MAX_DATE);
            start..=end
        } else if self.year.is_some() || self.month.is_some() {
            let today = Utc::today().naive_local();
            let year = self.year.unwrap_or_else(|| today.year());
            let month = self.month.unwrap_or_else(|| today.month());
            let end_of_month = match month {
                12 => NaiveDate::from_ymd(year + 1, month, 1).pred(),
                _ => NaiveDate::from_ymd(year, month + 1, 1).pred(),
            };
            NaiveDate::from_ymd(year, month, 1)..=end_of_month
        } else {
            chrono::naive::MIN_DATE..=chrono::naive::MAX_DATE
        }
    }
}
