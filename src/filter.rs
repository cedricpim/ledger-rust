use std::ops::RangeInclusive;

use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};

#[derive(Debug)]
pub struct Filter {
    year: Option<i32>,
    month: Option<u32>,
    from: Option<Date>,
    till: Option<Date>,
    categories: Vec<String>,
}

impl Filter {
    pub fn new(
        year: Option<i32>,
        month: Option<u32>,
        from: Option<Date>,
        till: Option<Date>,
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

    fn period(&self) -> RangeInclusive<Date> {
        let (start, end) = if self.year.is_some() || self.month.is_some() {
            let today = Date::today();
            let year = self.year.unwrap_or_else(|| today.year());
            let month = self.month.unwrap_or_else(|| today.month());
            let start: Date = chrono::naive::NaiveDate::from_ymd(year, month, 1).into();
            (start, start.end_of_month())
        } else {
            (
                self.from.unwrap_or_else(|| chrono::naive::MIN_DATE.into()),
                self.till.unwrap_or_else(|| chrono::naive::MAX_DATE.into()),
            )
        };
        start..=end
    }
}
