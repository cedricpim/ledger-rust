use std::ops::RangeInclusive;

use crate::cmd::{balance, report, show};
use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};

#[derive(Clone, Debug, Default)]
pub struct Filter {
    pub start: Option<Date>,
    pub end: Option<Date>,
    categories: Vec<String>,
    excluded_categories: Vec<String>,
    transfer: String,
    ignored_accounts: Vec<String>,
    investments: String,
}

impl Filter {
    pub fn show(args: &show::Args) -> Self {
        let (start, end) = Self::bounds(
            args.flag_year,
            args.flag_month,
            args.flag_from,
            args.flag_till,
        );

        Self {
            start,
            end,
            categories: args.flag_categories.clone(),
            ..Default::default()
        }
    }

    pub fn balance(args: &balance::Args) -> Self {
        let (start, end) = Self::bounds(None, None, None, args.flag_date);

        Self {
            start,
            end,
            ..Default::default()
        }
    }

    pub fn report(args: &report::Args, config: &Config) -> Self {
        let today = Date::today();

        let (start, end) = Self::bounds(
            Some(args.flag_year.unwrap_or_else(|| today.year())),
            Some(args.flag_month.unwrap_or_else(|| today.month())),
            args.flag_from,
            args.flag_till,
        );

        Self {
            start,
            end,
            excluded_categories: args.flag_exclude.clone(),
            transfer: config.transfer.clone(),
            ignored_accounts: config.ignored_accounts.clone(),
            investments: config.investments.clone(),
            ..Default::default()
        }
    }

    pub fn total(config: &Config, end: Option<Date>) -> Self {
        Self {
            end,
            ignored_accounts: config.ignored_accounts.clone(),
            ..Default::default()
        }
    }

    pub fn push(config: &Config) -> Self {
        Self {
            ignored_accounts: config.ignored_accounts.clone(),
            ..Default::default()
        }
    }

    pub fn networth(config: &Config) -> Self {
        Self {
            ignored_accounts: config.ignored_accounts.clone(),
            investments: config.investments.clone(),
            ..Default::default()
        }
    }

    pub fn excluded(&self, value: &str) -> bool {
        Filter::with(&value, &self.excluded_categories)
    }

    pub fn accountable(&self, value: &str) -> bool {
        !Filter::with(&value, &self.ignored_accounts)
    }

    pub fn transfer(&self, value: &str) -> bool {
        value == self.transfer
    }

    pub fn investment(&self, value: &str) -> bool {
        value == self.investments
    }

    pub fn within(&self, date: Date) -> bool {
        self.period().contains(&date)
    }

    pub fn display(&self, line: &Line) -> bool {
        (self.categories.is_empty() || Filter::with(&line.category(), &self.categories))
            && self.within(line.date())
    }

    fn period(&self) -> RangeInclusive<Date> {
        let lower = self.start.unwrap_or_else(|| chrono::naive::MIN_DATE.into());
        let upper = self.end.unwrap_or_else(|| chrono::naive::MAX_DATE.into());

        lower..=upper
    }

    fn with(value: &str, list: &[String]) -> bool {
        let values: Vec<String> = list.iter().map(|v| v.to_uppercase()).collect();

        values.contains(&value.to_uppercase())
    }

    fn bounds(
        year: Option<i32>,
        month: Option<u32>,
        from: Option<Date>,
        till: Option<Date>,
    ) -> (Option<Date>, Option<Date>) {
        if (year.is_some() || month.is_some()) && from.is_none() && till.is_none() {
            let today = Date::today();
            let selected_year = year.unwrap_or_else(|| today.year());
            let selected_month = month.unwrap_or_else(|| today.month());
            let start = Date::from_ymd(selected_year, selected_month, 1);
            (Some(start), Some(start.end_of_month()))
        } else {
            (from, till)
        }
    }
}
