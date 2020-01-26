use std::ops::RangeInclusive;

use crate::cmd::{balance, report, show};
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
    ignored_categories: Vec<String>,
    ignored_accounts: Vec<String>,
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

    pub fn report(args: &report::Args, config: &Config) -> Self {
        let today = Date::today();

        Self {
            year: Some(args.flag_year.unwrap_or_else(|| today.year())),
            month: Some(args.flag_month.unwrap_or_else(|| today.month())),
            from: args.flag_from,
            till: args.flag_till,
            excluded_categories: args.flag_exclude.clone(),
            ignored_categories: config.ignored_categories.clone(),
            ignored_accounts: config.ignored_accounts.clone(),
            ..Default::default()
        }
    }

    pub fn totals(config: &Config) -> Self {
        Self {
            ignored_accounts: config.ignored_accounts.clone(),
            ..Default::default()
        }
    }

    pub fn apply(&self, line: &Line) -> bool {
        Filter::with(&line.category(), &self.categories)
            && Filter::without(&line.category(), &self.ignored_categories)
            && Filter::without(&line.account(), &self.ignored_accounts)
            && self.within(line.date())
    }

    pub fn excluded(&self, value: &str) -> bool {
        !Filter::without(&value, &self.excluded_categories)
    }

    pub fn check(&self, value: &str) -> bool {
        Filter::with(&value, &self.categories)
            && Filter::without(&value, &self.ignored_categories)
            && Filter::without(&value, &self.ignored_accounts)
    }

    fn with(value: &str, list: &[String]) -> bool {
        let values: Vec<String> = list.iter().map(|v| v.to_uppercase()).collect();

        values.is_empty() || values.contains(&value.to_uppercase())
    }

    fn without(value: &str, list: &[String]) -> bool {
        let values: Vec<String> = list.iter().map(|v| v.to_uppercase()).collect();

        !values.contains(&value.to_uppercase())
    }

    fn within(&self, date: Date) -> bool {
        self.period().contains(&date)
    }

    fn period(&self) -> RangeInclusive<Date> {
        let (start, end) = if (self.year.is_some() || self.month.is_some()) && self.from.is_none() && self.till.is_none() {
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
