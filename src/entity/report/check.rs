use prettytable::{format, Cell, Row, Table};

use std::cmp::Ordering;
use std::ops::AddAssign;

use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::filter::Filter;
use crate::resource::Resource;
use crate::{CliResult, Mode};

#[derive(Default)]
pub struct Report {
    items: Vec<Item>,
    current: Option<Item>,
}

impl Report {
    fn title() -> Row {
        Row::new(vec![Cell::new("Report").with_hspan(4).style_spec("bcFC")])
    }

    fn headers() -> Row {
        Row::new(vec![
            Cell::new("Account").style_spec("bcFB"),
            Cell::new("Date").style_spec("bcFB"),
            Cell::new("Identifier").style_spec("bcFB"),
            Cell::new("Amount").style_spec("bFB"),
        ])
    }

    pub fn new(config: &Config, filter: &Filter) -> CliResult<Report> {
        let mut report = Self::default();

        let mut resource = Resource::new(config, Mode::Ledger)?;

        resource.line(&mut |record| {
            if !filter.within(record.date()) || filter.investment(&record.category()) {
                return Ok(());
            }

            let item: Item = record.into();

            let current_item = match report.current.take() {
                None => item,
                Some(mut existing_item) => {
                    if existing_item == item {
                        existing_item += item;
                        existing_item
                    } else {
                        report.items.push(existing_item);
                        item
                    }
                }
            };

            report.current = Some(current_item);

            Ok(())
        })?;

        if let Some(current_item) = report.current.take() {
            report.items.push(current_item);
        };

        report.items.sort();

        Ok(report)
    }

    pub fn display(&self) {
        let mut table = Table::new();

        table.set_format(format::FormatBuilder::new().padding(2, 3).build());

        table.set_titles(Report::title());

        table.add_row(Report::headers());

        for item in self.items.iter() {
            table.add_row(item.row());
        }

        table.printstd();
    }
}

#[derive(Default)]
struct Item {
    account: String,
    date: Date,
    identifier: String,
    amount: Money,
}

impl From<&mut crate::entity::line::Line> for Item {
    fn from(line: &mut Line) -> Item {
        let identifier = if line.venue().is_empty() {
            line.category()
        } else {
            line.venue()
        };

        Self {
            account: line.account(),
            date: line.date(),
            identifier,
            amount: line.amount(),
        }
    }
}

impl Eq for Item {}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.account == other.account
            && self.date == other.date
            && self.identifier == other.identifier
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.account.cmp(&other.account) {
            Ordering::Equal => self.date.cmp(&self.date),
            val => val,
        }
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AddAssign for Item {
    fn add_assign(&mut self, other: Self) {
        self.amount += other.amount;
    }
}

impl Item {
    fn row(&self) -> Row {
        Row::new(vec![
            Cell::new(&self.account).style_spec("bFW"),
            Cell::new(&self.date.to_string()).style_spec("bFW"),
            Cell::new(&self.identifier).style_spec("bFW"),
            Cell::new(&format!("{}", self.amount)).style_spec("bFW"),
        ])
    }
}
