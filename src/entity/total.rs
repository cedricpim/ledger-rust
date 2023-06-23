use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::{Currency, Money};
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::{util, CliResult};

#[derive(Debug)]
pub struct Total {
    value: i64,
    currency: Currency,
    filter: Filter,
}

impl Total {
    pub fn new(currency: Option<&String>, config: &Config, date: Option<Date>) -> CliResult<Self> {
        Ok(Self {
            value: 0,
            currency: util::currency(currency, config)?,
            filter: Filter::total(config, date),
        })
    }

    pub fn sum(&mut self, record: &Line, exchange: &Exchange) -> CliResult<()> {
        let exchanged = record.exchange(self.currency, exchange)?;

        if self.filter.accountable(&record.account()) && self.filter.within(record.date()) {
            self.value += exchanged.amount().cents();
        };

        Ok(())
    }

    pub fn amount(&self) -> Money {
        Money::new(self.currency, self.value)
    }
}
