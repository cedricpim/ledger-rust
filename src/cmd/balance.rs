use clap::Clap;
use prettytable::{format, Cell, Row, Table};

use std::collections::BTreeMap;
use std::ops::AddAssign;

use crate::config::Config;
use crate::entity::{date::Date, line::Line, line::Liner, money::Money, total::Total};
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::resource::Resource;
use crate::{util, CliResult, Mode};

#[derive(Clap, Debug)]
pub struct Args {
    /// Display all accounts
    #[clap(short, long)]
    all: bool,
    /// Calculate the current balance at a given date
    #[clap(short, long)]
    pub date: Option<Date>,
}

pub fn run(args: Args) -> CliResult<()> {
    let config = Config::new()?;

    args.calculate(&config)
}

impl Args {
    fn calculate(&self, config: &Config) -> CliResult<()> {
        let exchange = Exchange::new(&config)?;

        let filter = Filter::balance(&self);

        let mut total = Total::new(Some(&config.currency), &config, filter.end)?;

        let report = Report::new(&mut total, &config, &exchange, &filter)?;

        let summary = Summary::new(total);

        report.display(&self);

        summary.display();

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
        total: &mut Total,
        config: &Config,
        exchange: &Exchange,
        filter: &Filter,
    ) -> CliResult<Report> {
        let mut report = Self {
            items: BTreeMap::new(),
        };

        let mut resource = Resource::new(&config, Mode::Ledger)?;

        resource.line(&mut |record| {
            total.sum(record, &exchange)?;

            if !filter.within(record.date()) {
                return Ok(());
            }

            report.add(Item::new(&record));

            Ok(())
        })?;

        Ok(report)
    }

    fn add(&mut self, item: Item) {
        self.items
            .entry(item.account.to_string())
            .and_modify(|i| *i += item.clone())
            .or_insert(item);
    }

    fn display(&self, args: &Args) {
        let mut table = Table::new();

        table.set_format(format::FormatBuilder::new().padding(3, 5).build());

        table.set_titles(Report::title());

        table.add_row(Report::headers());

        for item in self.items.values() {
            if args.all || !item.value.zero() {
                table.add_row(item.row());
            }
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

#[derive(Debug)]
struct Summary {
    total: Total,
}

impl Summary {
    fn title() -> Row {
        Row::new(vec![Cell::new("Totals").style_spec("bcFC")])
    }

    fn new(total: Total) -> Self {
        Self { total }
    }

    fn row(&self) -> Row {
        Row::new(vec![
            Cell::new(&format!("{}", self.total.amount())).style_spec("brFB")
        ])
    }

    pub fn display(self) {
        let mut table = Table::new();

        table.set_format(
            format::FormatBuilder::new()
                .separators(
                    &[format::LinePosition::Top],
                    format::LineSeparator::new('─', '┬', '┌', '┐'),
                )
                .padding(15, 10)
                .build(),
        );

        table.set_titles(Summary::title());

        table.add_row(self.row());

        table.printstd();
    }
}
