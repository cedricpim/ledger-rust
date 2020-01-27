use prettytable::{format, Cell, Row, Table};

use crate::filter::Filter;
use crate::exchange::Exchange;
use crate::config::Config;
use crate::entity::{line::Line, line::Liner};
use crate::entity::money::{Currency, Money};
use crate::{util, CliResult};

#[derive(Debug)]
pub struct Total {
    value: i64,
    currency: Currency,
    filter: Filter,
}

impl Total {
    fn title() -> Row {
        Row::new(vec![
            Cell::new("Totals").style_spec("bcFC")
        ])
    }

    pub fn new(currency: &str, config: &Config) -> CliResult<Self> {
        Ok(Self {
            value: 0,
            currency: util::currency(currency, &config)?,
            filter: Filter::total(&config),
        })
    }

    pub fn sum(&mut self, record: &Line, exchange: &Exchange) -> CliResult<()> {
        let exchanged = record.exchange(self.currency, &exchange)?;

        if self.filter.ignore_account(&record.account()) {
            self.value += exchanged.amount().cents();
        };

        Ok(())
    }

    pub fn display(self) {
        let mut table = Table::new();

        table.set_format(
            format::FormatBuilder::new()
                .separators(
                    &[format::LinePosition::Top],
                    format::LineSeparator::new('─', '┬', '┌', '┐'),
                )
                .padding(10, 10)
                .build(),
        );

        table.set_titles(Total::title());

        table.add_row(self.row());

        table.printstd();
    }

    fn row(&self) -> Row {
        Row::new(vec![
            Cell::new(&format!("{}", self.amount())).style_spec("brFB")
        ])
    }

    fn amount(&self) -> Money {
        steel_cent::Money::of_minor(self.currency.into(), self.value).into()
    }
}
