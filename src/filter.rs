use std::ops::RangeInclusive;

use crate::cmd::{balance, show};
use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};

#[derive(Debug, Default)]
pub struct Filter {
    year: Option<i32>,
    month: Option<u32>,
    from: Option<Date>,
    till: Option<Date>,
    categories: Vec<String>,
    excluded_categories: Vec<String>,
    excluded_accounts: Vec<String>,
}

impl Filter {
    pub fn show(args: &show::Args) -> Self {
        Self {
            year: args.flag_year,
            month: args.flag_month,
            from: args.flag_from,
            till: args.flag_till,
            categories: args.flag_categories.clone(),
            ..Default::default()
        }
    }

    pub fn balance(args: &balance::Args) -> Self {
        Self {
            from: args.flag_date,
            ..Default::default()
        }
    }

    pub fn totals(config: &Config) -> Self {
        Self {
            excluded_accounts: config.accounts.clone(),
            ..Default::default()
        }
    }

    pub fn apply(&self, line: &Line) -> bool {
        self.with(&line.category(), &self.categories)
            && Filter::without(&line.category(), &self.excluded_categories)
            && Filter::without(&line.account(), &self.excluded_accounts)
            && self.within(line.date())
    }

    pub fn check(&self, value: &str) -> bool {
        self.with(&value, &self.categories)
            && Filter::without(&value, &self.excluded_categories)
            && Filter::without(&value, &self.excluded_accounts)
    }

    fn with(&self, value: &str, list: &[String]) -> bool {
        let values: Vec<String> = list.iter().map(|v| v.to_uppercase()).collect();

        self.categories.is_empty() || values.contains(&value.to_uppercase())
    }

    fn without(value: &str, list: &[String]) -> bool {
        let values: Vec<String> = list.iter().map(|v| v.to_uppercase()).collect();

        !values.contains(&value.to_uppercase())
    }

    fn within(&self, date: Date) -> bool {
        self.period().contains(&date)
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
