use prettytable::format::{Alignment, FormatBuilder};
use prettytable::{color, Attr, Cell, Row, Table};

use std::collections::BTreeMap;

use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::{Currency, Money};
use crate::entity::networth::Networth;
use crate::exchange::Exchange;
use crate::resource::Resource;
use crate::{util, CliResult, Mode};

#[derive(Debug)]
pub struct Report {
    networth: Networth,
    exchange: Exchange,
    config: Config,
}

impl Report {
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

    pub fn new(config: Config, exchange: Exchange, currency: Currency) -> CliResult<Report> {
        Ok(Self {
            networth: Networth::new(&config, &exchange, currency)?,
            exchange,
            config,
        })
    }

    pub fn save(&self) -> CliResult<()> {
        let resource = Resource::new(&self.config, Mode::Networth)?;

        let entries = self.entries(&resource)?;

        resource.apply(|file| {
            let mut wtr = csv::WriterBuilder::new().from_path(file.path())?;

            for entry in entries.values() {
                entry.write(&mut wtr)?;
            }

            wtr.flush()?;

            Ok(())
        })?;

        Ok(())
    }

    pub fn display(&self) {
        let mut table = Table::new();

        table.set_format(FormatBuilder::new().padding(0, 3).build());

        table.set_titles(Report::title());

        table.add_row(Report::headers());

        for investment in self.networth.investments.values() {
            if investment.value().zero() {
                continue;
            };

            let color = Attr::ForegroundColor(color::BRIGHT_WHITE);

            table.add_row(Row::new(vec![
                Cell::new(&investment.name())
                    .with_style(Attr::Bold)
                    .with_style(color),
                util::money_cell(&investment.value(), true, false, Alignment::LEFT)
                    .with_style(color),
                util::percentage_cell(&investment.value(), &self.networth.total(), Alignment::LEFT)
                    .with_style(color),
            ]));
        }

        let color = Attr::ForegroundColor(color::BRIGHT_CYAN);

        let cash = self.networth.current_on(Date::today());

        table.add_row(Row::new(vec![
            Cell::new(&"Cash").with_style(Attr::Bold).with_style(color),
            util::money_cell(&cash, true, false, Alignment::LEFT).with_style(color),
            util::percentage_cell(&cash, &self.networth.total(), Alignment::LEFT).with_style(color),
        ]));

        table.add_row(self.row());

        table.printstd();
    }

    fn entries(&self, resource: &Resource) -> CliResult<BTreeMap<Date, Line>> {
        let mut result: BTreeMap<Date, Line> = BTreeMap::new();

        resource.line(&mut |record| {
            let mut exchanged = record.exchange(self.networth.currency, &self.exchange)?;

            exchanged.set_invested(self.networth.invested_on(exchanged.date()));
            exchanged
                .set_amount(self.networth.current_on(exchanged.date()) + exchanged.investment());

            result.entry(exchanged.date()).or_insert(exchanged);

            Ok(())
        })?;

        let current = self.networth.current();

        result.entry(current.date()).or_insert(current);

        Ok(result)
    }

    fn row(&self) -> Row {
        let color = Attr::ForegroundColor(color::BRIGHT_YELLOW);

        let money = Money::new(self.networth.currency, 1);

        Row::new(vec![
            Cell::new(&"Total").with_style(Attr::Bold).with_style(color),
            util::money_cell(&self.networth.total(), true, false, Alignment::LEFT)
                .with_style(color),
            util::percentage_cell(&money, &money, Alignment::LEFT).with_style(color),
        ])
    }
}
