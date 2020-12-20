use prettytable::{format, Cell, Row, Table};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::AddAssign;

use crate::cmd::report::Args;
use crate::config::Config;
use crate::entity::line::{Line, Liner};
use crate::entity::money::{Currency, Money};
use crate::entity::total::Total;
use crate::exchange::Exchange;
use crate::filter::Filter;
use crate::resource::Resource;
use crate::{util, CliResult, Mode};

#[derive(Default, Debug, Clone)]
pub struct Report {
    currency: Currency,
    expense: i64,
    income: i64,
    excluded: i64,
    items: HashMap<String, Item>,
    occurrences: u32,
    total: i64,
    previous: Option<Line>,
}

impl Report {
    fn title() -> Row {
        Row::new(vec![Cell::new("Report").with_hspan(4).style_spec("bcFC")])
    }

    fn headers() -> Row {
        Row::new(vec![
            Cell::new("Category").style_spec("bcFB").with_hspan(2),
            Cell::new("Amount").style_spec("bFB"),
            Cell::new("(%)").style_spec("bFB"),
        ])
    }

    pub fn new(
        args: &Args,
        total: &mut Total,
        config: &Config,
        exchange: &Exchange,
        filter: &Filter,
    ) -> CliResult<Report> {
        let mut report = Self {
            currency: util::currency(args.currency.as_ref(), &config)?,
            ..Default::default()
        };

        let mut resource = Resource::new(&config, Mode::Ledger)?;

        resource.line(&mut |record| {
            total.sum(record, &exchange)?;

            if !filter.within(record.date()) || filter.investment(&record.category()) {
                return Ok(());
            }

            if filter.transfer(&record.category()) {
                match report.previous.take() {
                    None => report.previous = Some(record.clone()),
                    Some(mut val) => {
                        if filter.accountable(&record.account())
                            ^ filter.accountable(&val.account())
                        {
                            // Set the category as the destination/source account to not show all
                            // transfers with the default category for transfers.
                            report.process(record, val.account(), &filter, &exchange)?;
                            report.process(&mut val, record.account(), &filter, &exchange)?;
                        }
                    }
                }
            } else {
                report.process(record, record.category(), &filter, &exchange)?;
            };

            Ok(())
        })?;

        Ok(report)
    }

    pub fn display(&self) {
        let mut table = Table::new();

        table.set_format(format::FormatBuilder::new().padding(2, 3).build());

        table.set_titles(Report::title());

        table.add_row(Report::headers());

        for item in self.sorted() {
            table.add_row(item.row(&self));
        }

        table.add_row(self.row());

        table.printstd();
    }

    fn process(
        &mut self,
        record: &mut Line,
        category: String,
        filter: &Filter,
        exchange: &Exchange,
    ) -> CliResult<()> {
        if !filter.accountable(&record.account()) {
            return Ok(());
        };

        let exchanged = record.exchange(self.currency, &exchange)?;

        if filter.excluded(&category) {
            self.excluded += exchanged.amount().cents();

            return Ok(());
        };

        self.add(Item::new(exchanged.amount(), category));

        Ok(())
    }

    fn add(&mut self, item: Item) {
        if item.value.positive() {
            self.income += item.value.cents();
        } else {
            self.expense += item.value.cents();
        };

        self.total += item.value.cents();
        self.occurrences += 1;

        self.items
            .entry(item.category.to_string())
            .and_modify(|i| *i += item.clone())
            .or_insert(item);
    }

    fn sorted(&self) -> Vec<Item> {
        let mut values: Vec<Item> = self.items.values().cloned().collect();

        values.sort();

        values
    }

    fn total(&self) -> Money {
        steel_cent::Money::of_minor(self.currency.into(), self.total).into()
    }

    fn percentage(&self) -> f64 {
        let expense = self.expense.abs();

        if self.income == 0 {
            100.0
        } else if self.income > expense {
            ((self.income - expense) as f64) / (self.income as f64) * 100.0
        } else {
            ((expense - self.income) as f64) / (self.income as f64) * 100.0
        }
    }

    fn row(&self) -> Row {
        Row::new(vec![
            Cell::new(&format!("({})", self.occurrences)).style_spec("bFY"),
            Cell::new(&"Total").style_spec("bFY"),
            Cell::new(&format!("{}", self.total())).style_spec("bFY"),
            Cell::new(&format!("{:.2}", self.percentage())).style_spec("bFY"),
        ])
    }
}

#[derive(Debug, Clone)]
struct Item {
    category: String,
    value: Money,
    occurrences: u32,
}

impl Item {
    fn new(value: Money, category: String) -> Self {
        Self {
            category,
            value,
            occurrences: 1,
        }
    }

    fn percentage(&self, report: &Report) -> f64 {
        if self.value.positive() {
            (self.value.cents() as f64) / (report.income as f64) * 100.0
        } else {
            (self.value.cents() as f64) / (report.expense as f64) * 100.0
        }
    }

    fn row(&self, report: &Report) -> Row {
        Row::new(vec![
            Cell::new(&format!("({})", self.occurrences)).style_spec("bFW"),
            Cell::new(&self.category).style_spec("bFW"),
            Cell::new(&format!("{}", self.value)).style_spec("bFW"),
            Cell::new(&format!("{:.2}", self.percentage(&report))).style_spec("bFW"),
        ])
    }
}

impl AddAssign for Item {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            category: self.category.to_string(),
            value: self.value + other.value,
            occurrences: self.occurrences + 1,
        }
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        other.value.cmp(&self.value)
    }
}

impl Eq for Item {}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.value.abs() == other.value.abs()
    }
}

#[derive(Debug)]
pub struct Summary {
    currency: Currency,
    expense: i64,
    income: i64,
    excluded: i64,
    total: Total,
}

impl Summary {
    fn title() -> Row {
        Row::new(vec![Cell::new("Totals").with_hspan(9).style_spec("bcFC")])
    }

    pub fn new(report: &Report, total: Total) -> Self {
        Self {
            currency: report.currency,
            expense: report.expense,
            income: report.income,
            excluded: report.excluded,
            total,
        }
    }

    pub fn display(&self) {
        let mut table = Table::new();

        table.set_format(
            format::FormatBuilder::new()
                .separators(
                    &[format::LinePosition::Top],
                    format::LineSeparator::new('─', '┬', '┌', '┐'),
                )
                .padding(1, 2)
                .build(),
        );

        table.set_titles(Summary::title());

        table.add_row(self.row());

        table.add_row(self.total());

        table.printstd();
    }

    fn income(&self) -> Money {
        let value = if self.excluded < 0 && self.income > self.excluded.abs() {
            self.income + self.excluded
        } else {
            self.income
        };

        steel_cent::Money::of_minor(self.currency.into(), value).into()
    }

    fn expense(&self) -> Money {
        steel_cent::Money::of_minor(self.currency.into(), self.expense).into()
    }

    fn difference(&self) -> Money {
        self.income() - self.expense().abs()
    }

    fn row(&self) -> Row {
        Row::new(vec![
            util::money_cell(&self.income(), false, false, format::Alignment::RIGHT).with_hspan(3),
            util::money_cell(&self.expense(), false, false, format::Alignment::LEFT).with_hspan(2),
            util::money_cell(&self.difference(), false, true, format::Alignment::LEFT)
                .with_hspan(3),
            util::percentage_cell(&self.difference(), &self.income(), format::Alignment::LEFT),
        ])
    }

    fn total(&self) -> Row {
        Row::new(vec![
            Cell::new(&format!("{}", self.total.amount()))
                .style_spec("bcFB")
                .with_hspan(8),
            util::percentage_cell(
                &self.difference(),
                &self.total.amount(),
                format::Alignment::LEFT,
            ),
        ])
    }
}
