use std::collections::{BTreeMap, HashMap};
use std::ops::AddAssign;

use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::entry::Entry;
use crate::entity::line::{Line, Liner};
use crate::entity::money::{Currency, Money};
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::resource::Resource;
use crate::service::justetf::Asset;
use crate::Mode;

#[derive(Debug)]
pub struct Networth {
    pub currency: Currency,
    pub invested: HashMap<Date, Money>,
    pub investments: BTreeMap<String, Investment>,
    pub current: HashMap<Date, Money>,
    cash: Money,
}

impl Networth {
    pub fn new(
        config: &Config,
        exchange: &Exchange,
        currency: Currency,
    ) -> anyhow::Result<Networth> {
        let mut networth = Self {
            currency,
            cash: Money::new(currency, 0),
            invested: HashMap::new(),
            investments: BTreeMap::new(),
            current: HashMap::new(),
        };

        let mut resource = Resource::new(config, Mode::Ledger)?;

        let filter = Filter::networth(config);

        resource.line(&mut |record| {
            if filter.accountable(&record.account()) {
                networth.add(record, &filter, exchange)?;
            };

            Ok(())
        })?;

        Ok(networth)
    }

    pub fn total(&self) -> Money {
        self.current_on(Date::today()) + self.investments()
    }

    pub fn invested_on(&self, date: Date) -> Money {
        self.invested
            .get(&date)
            .unwrap_or(&Money::new(self.currency, 0))
            .to_owned()
    }

    pub fn current_on(&self, date: Date) -> Money {
        let mut available_date = date;

        while !self.current.contains_key(&available_date)
            && date.since(available_date).num_days() < 30
        {
            available_date = available_date.pred();
        }

        self.current
            .get(&available_date)
            .unwrap_or(&Money::new(self.currency, 0))
            .to_owned()
    }

    pub fn current(&self) -> Line {
        let today = Date::today();
        let investment = self.investments();

        Entry {
            date: today,
            invested: self.invested_on(today),
            amount: self.current_on(today) + self.investments(),
            currency: self.currency,
            investment,
            id: "".to_string(),
        }
        .into()
    }

    fn add(&mut self, record: &Line, filter: &Filter, exchange: &Exchange) -> anyhow::Result<()> {
        let exchanged = record.exchange(self.currency, exchange)?;

        self.cash += exchanged.amount();

        let cash = self.cash;

        self.current
            .entry(exchanged.date())
            .and_modify(|i| *i = cash)
            .or_insert_with(|| cash);

        if filter.investment(&exchanged.category()) {
            let currency = self.currency;

            self.investments
                .entry(exchanged.description())
                .and_modify(|i| *i += exchanged.clone())
                .or_insert_with(|| Investment::new(&exchanged, currency));

            self.invested
                .entry(exchanged.date())
                .and_modify(|i| *i += exchanged.amount() * -1)
                .or_insert_with(|| exchanged.amount() * -1);
        }

        Ok(())
    }

    fn investments(&self) -> Money {
        self.investments
            .values()
            .fold(Money::new(self.currency, 0), |acc, investment| {
                acc + investment.value()
            })
    }
}

#[derive(Debug, Clone)]
pub struct Investment {
    pub code: String,
    pub spent: Money,
    pub quantity: i64,
    pub currency: Currency,
    pub asset: Asset,
    pub price: Money,
}

impl Investment {
    pub fn new(record: &Line, currency: Currency) -> Self {
        let asset = Asset::download(&record.description(), &currency)
            .unwrap_or_else(|e| crate::werr!(1, "{}", e));

        let quantity = record
            .quantity()
            .parse::<i64>()
            .unwrap_or_else(|e| crate::werr!(1, "{}", e));

        Self {
            code: record.description(),
            spent: record.amount(),
            price: asset.quote,
            currency,
            quantity,
            asset,
        }
    }

    pub fn value(&self) -> Money {
        self.price * self.quantity
    }

    pub fn name(&self) -> String {
        self.asset.name.to_string()
    }
}

impl AddAssign<Line> for Investment {
    fn add_assign(&mut self, other: Line) {
        let quantity = other
            .quantity()
            .parse::<i64>()
            .unwrap_or_else(|e| crate::werr!(1, "{}", e));

        *self = Self {
            code: self.code.to_string(),
            spent: self.spent + other.amount(),
            quantity: self.quantity + quantity,
            currency: self.currency,
            asset: self.asset.clone(),
            price: self.price,
        }
    }
}
