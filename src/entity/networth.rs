use prettytable::{color, Attr, Cell, Row};
use prettytable::format::Alignment;

use std::ops::AddAssign;

use crate::entity::line::{Line, Liner};
use crate::entity::money::{Currency, Money};
use crate::service::justetf::Asset;
use crate::exchange::Exchange;
use crate::error::CliError;
use crate::{util, werr};

#[derive(Debug)]
pub struct Cash {
    pub amount: Money,
}

impl Cash {
    pub fn row(&self, total: &Money) -> Row {
        let color = Attr::ForegroundColor(color::BRIGHT_RED);

        Row::new(vec![
            Cell::new(&"Cash").with_style(Attr::Bold).with_style(color),
            util::money_cell(&self.amount, true, false, Alignment::LEFT).with_style(color),
            util::percentage_cell(self.percentage(&total), Alignment::LEFT).with_style(color),
        ])
    }

    fn percentage(&self, total: &Money) -> f64 {
        (self.amount.cents() as f64 / total.cents() as f64) * 100.0
    }
}

impl AddAssign for Cash {
    fn add_assign(&mut self, other: Self) {
        *self = Self { amount: self.amount + other.amount }
    }
}

#[derive(Debug, Clone)]
pub struct Investment {
    pub code: String,
    pub spent: Money,
    pub quantity: u64,
    pub currency: Currency,
    pub exchange: Exchange,
    pub asset: Asset,
}

impl Investment {
    pub fn new(record: &Line, currency: &Currency, exchange: &Exchange) -> Self {
        let asset = Asset::download(&record.description()).unwrap_or_else(|e| werr!(1, "{}", e));

        let quantity = record.quantity().parse::<u64>().map_err(CliError::from).unwrap_or_else(|e| werr!(1, "{}", e));

        Self {
            code: record.description(),
            spent: record.amount(),
            currency: currency.clone(),
            exchange: exchange.clone(),
            quantity: quantity,
            asset: asset,
        }
    }

    pub fn row(&self, total: &Money) -> Row {
        let color = Attr::ForegroundColor(color::BRIGHT_WHITE);

        Row::new(vec![
            Cell::new(&self.name()).with_style(Attr::Bold).with_style(color),
            util::money_cell(&self.value(), true, false, Alignment::LEFT).with_style(color),
            util::percentage_cell(self.percentage(&total), Alignment::LEFT).with_style(color),
        ])
    }

    pub fn value(&self) -> Money {
        self.price() * self.quantity
    }

    fn name(&self) -> String {
        self.asset.name.to_string()
    }

    fn percentage(&self, total: &Money) -> f64 {
        (self.value().cents() as f64) / (total.cents() as f64) * 100.0
    }

    fn price(&self) -> Money {
        let currency = Currency::parse(&self.asset.currency).unwrap_or_else(|e| werr!(1, "{}", e));

        let money = Money::parse(&self.asset.value, currency).unwrap_or_else(|e| werr!(1, "{}", e));

        money.exchange(self.currency, &self.exchange).unwrap_or_else(|e| werr!(1, "{}", e))
    }
}

impl AddAssign<Line> for Investment {
    fn add_assign(&mut self, other: Line) {
        let quantity = other.quantity().parse::<u64>().map_err(CliError::from).unwrap_or_else(|e| werr!(1, "{}", e));

        *self = Self {
            code: self.code.to_string(),
            spent: self.spent + other.amount(),
            quantity: self.quantity + quantity,
            currency: self.currency,
            exchange: self.exchange.clone(),
            asset: self.asset.clone(),
        }
    }
}
