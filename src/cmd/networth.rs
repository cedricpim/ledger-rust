use prettytable::{color, Attr, Cell, Row, Table};
use prettytable::format::{Alignment, FormatBuilder};
use serde::Deserialize;

use std::collections::BTreeMap;

use crate::config::Config;
use crate::entity::line::{Line, Liner};
use crate::entity::money::{Currency, Money};
use crate::entity::networth::{Cash, Investment};
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::repository::Resource;
use crate::{util, CliResult};

static USAGE: & str = "
Calculate the current networth.
Shows list of entries that match the filters.

This command will print the list of the current networth, per asset. If the storage option is
provided, then the total amount of the current networth is stored in the networth CSV as a new
entry.

Usage:
    ledger networth [options]

Options:
    -C, --currency=<currency>           Display entries on the currency
    -S, --save                          Save the total networth to the networth CSV
    -h, --help                          Display this message
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_currency: String,
    flag_save: bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.generate(&config)
}

impl Args {
    fn generate(&self, config: &Config) -> CliResult<()> {
        let exchange = Exchange::new(&config)?;

        let currency = util::currency(&self.flag_currency, &config)?;

        let networth = Networth::new(&config, &exchange, currency)?;

        networth.display();

        Ok(())
    }
}

#[derive(Debug)]
struct Networth {
    currency: Currency,
    cash: Cash,
    investments: BTreeMap<String, Investment>,
}

impl Networth {
    fn title() -> Row {
        Row::new(vec![Cell::new("Networth").with_hspan(3).style_spec("bcFC")])
    }

    fn headers() -> Row {
        Row::new(vec![
            Cell::new("Description").style_spec("bcFB"),
            Cell::new("Amount").style_spec("bcFB"),
            Cell::new("(%)").style_spec("bcFB"),
        ])
    }

    fn new(config: &Config, exchange: &Exchange, currency: Currency) -> CliResult<Networth> {
        let mut networth = Self {
            currency,
            cash: Cash { amount: Money::new(currency, 0) },
            investments: BTreeMap::new(),
        };

        let resource = Resource::new(&config, false)?;

        let filter = Filter::networth(&config);

        resource.line(&mut |record| {
            if !filter.apply(&record) {
                return Ok(());
            };

            networth.add(&record, &config, &exchange)?;

            Ok(())
        })?;

        Ok(networth)
    }

    fn add(&mut self, record: &Line, config: &Config, exchange: &Exchange) -> CliResult<()> {
        let exchanged = record.exchange(self.currency, &exchange)?;

        self.cash += Cash { amount: exchanged.amount() };

        if config.investments.contains(&exchanged.category()) {
            let currency = self.currency;

            self.investments
                .entry(exchanged.description())
                .and_modify(|i| *i += exchanged.clone())
                .or_insert_with(|| Investment::new(&exchanged, &currency, &exchange));
        }

        Ok(())
    }

    fn total(&self) -> Money {
        self.investments.values().fold(self.cash.amount, |acc, investment| acc + investment.value())
    }

    fn row(&self) -> Row {
        let color = Attr::ForegroundColor(color::BRIGHT_YELLOW);

        Row::new(vec![
            Cell::new(&"Total").with_style(Attr::Bold).with_style(color),
            util::money_cell(&self.total(), true, false, Alignment::LEFT).with_style(color),
            util::percentage_cell(100.0, Alignment::LEFT).with_style(color),
        ])
    }

    fn display(&self) {
        let mut table = Table::new();

        table.set_format(FormatBuilder::new().padding(0, 3).build());

        table.set_titles(Networth::title());

        table.add_row(Networth::headers());

        for investment in self.investments.values() {
            table.add_row(investment.row(&self.total()));
        }

        table.add_row(self.cash.row(&self.total()));

        table.add_row(self.row());

        table.printstd();
    }
}
