use prettytable::{format, Cell, Row, Table};
use serde::Deserialize;

use std::collections::BTreeMap;
use std::ops::AddAssign;

use crate::config::Config;
use crate::entity::{date::Date, line::Line, line::Liner, money::Money, total::Total};
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::repository::Resource;
use crate::{util, CliResult};

static USAGE: &'static str = "
Calculate the current balances for each account.

This command will calculate the current balance of each account and display it.

Usage:
    ledger balance [options]

Options:
    -a, --all          Display all accounts
    -d, --date=<date>  Calculate the current balance at a given date
    -h, --help         Display this message
";

#[derive(Debug, Deserialize)]
pub struct Args {
    flag_all: bool,
    pub flag_date: Option<Date>,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;

    let config = Config::new()?;

    args.calculate(&config)
}

impl Args {
    fn calculate(&self, config: &Config) -> CliResult<()> {
        let exchange = Exchange::new(&config)?;

        let mut total = Total::new(&config.currency.to_string(), &config)?;

        let report = Report::new(&self, &mut total, &config, &exchange)?;

        report.display();

        total.display();

        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
struct Report {
    items: BTreeMap<String, Item>,
}

impl Report {
    fn title() -> Row {
        Row::new(vec![Cell::new("Balance").with_hspan(2).style_spec("bcFC")])
    }

    fn headers() -> Row {
        Row::new(vec![
            Cell::new("Account").style_spec("brFB"),
            Cell::new("Amount").style_spec("blFB"),
        ])
    }

    fn new(
        args: &Args,
        total: &mut Total,
        config: &Config,
        exchange: &Exchange,
    ) -> CliResult<Report> {
        let mut report = Self {
            items: BTreeMap::new(),
        };

        let resource = Resource::new(&config, false)?;

        let filter = Filter::balance(&args);

        resource.line(&mut |record| {
            total.sum(record, &exchange)?;

            if !filter.apply(&record) {
                return Ok(());
            }

            report.add(Item::new(&record));

            Ok(())
        })?;

        if !args.flag_all {
            for (account, item) in &report.items.clone() {
                if item.value.zero() {
                    report.items.remove(account);
                }
            }
        };

        Ok(report)
    }

    fn add(&mut self, item: Item) {
        self.items
            .entry(item.account.to_string())
            .and_modify(|i| *i += item.clone())
            .or_insert(item);
    }

    fn display(&self) {
        let mut table = Table::new();

        table.set_format(format::FormatBuilder::new().padding(3, 5).build());

        table.set_titles(Report::title());

        table.add_row(Report::headers());

        for (_account, item) in &self.items {
            table.add_row(item.row());
        }

        table.printstd();
    }
}

#[derive(Debug, Clone)]
struct Item {
    account: String,
    value: Money,
}

impl Item {
    fn new(record: &Line) -> Self {
        Self {
            account: record.account(),
            value: record.amount(),
        }
    }

    fn row(&self) -> Row {
        Row::new(vec![
            Cell::new(&self.account).style_spec("brFW"),
            util::money_cell(&self.value, false, false, format::Alignment::LEFT),
        ])
    }
}

impl AddAssign for Item {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            account: self.account.to_string(),
            value: self.value + other.value,
        }
    }
}
