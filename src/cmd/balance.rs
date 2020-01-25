use prettytable::{format, Cell, Row, Table};
use serde::Deserialize;

use std::collections::BTreeMap;

use crate::config::Config;
use crate::entity::{date::Date, line::Liner, money::Money};
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

        let balances: BTreeMap<String, Money> = self.balances(&config, &exchange)?;

        let totals: BTreeMap<String, Money> = Args::totals(&balances, &exchange, &config)?;

        Args::table(&balances);

        Args::table_total(&totals);

        Ok(())
    }

    fn balances(&self, config: &Config, exchange: &Exchange) -> CliResult<BTreeMap<String, Money>> {
        let resource = Resource::new(&config, false)?;

        let mut balances: BTreeMap<String, Money> = BTreeMap::new();

        let filter = Filter::balance(&self);

        resource.line(&mut |record| {
            if !filter.apply(&record) {
                return Ok(());
            }

            let value = match balances.get(&record.account()) {
                None => record.amount(),
                Some(val) => *val + record.exchange(Some(val.currency()), &exchange)?.amount(),
            };

            balances.insert(record.account(), value);

            Ok(())
        })?;

        if !self.flag_all {
            balances = balances.into_iter().filter(|&(_, v)| !v.zero()).collect();
        }

        Ok(balances)
    }

    fn totals(
        balances: &BTreeMap<String, Money>,
        exchange: &Exchange,
        config: &Config,
    ) -> CliResult<BTreeMap<String, Money>> {
        let mut totals: BTreeMap<String, Money> = BTreeMap::new();

        let filter = Filter::totals(&config);

        for value in balances.values() {
            let currency = value.currency();
            let result = match totals.get(&currency.code()) {
                None => {
                    let exchanged: CliResult<Vec<Money>> = balances
                        .iter()
                        .filter(|(k, _)| filter.check(&k))
                        .map(|(_, v)| v.exchange(Some(currency), &exchange))
                        .collect();
                    exchanged?
                        .iter()
                        .fold(Money::new(currency, 0), |acc, val| acc + *val)
                }
                Some(val) => *val,
            };

            totals.insert(currency.code(), result);
        }

        Ok(totals)
    }

    fn table(balances: &BTreeMap<String, Money>) {
        let mut table = Table::new();

        let headers = vec![
            Cell::new("Account").style_spec("brFB"),
            Cell::new("Amount").style_spec("blFB"),
        ];
        table.add_row(Row::new(headers));

        for (account, value) in balances.iter() {
            let format = if value.positive() {
                "blFG"
            } else if value.negative() {
                "blFR"
            } else {
                "blFD"
            };
            let row = vec![
                Cell::new(&account).style_spec("brFW"),
                Cell::new(&format!("{}", value)[1..]).style_spec(format),
            ];
            table.add_row(Row::new(row));
        }

        table.set_titles(Row::new(vec![Cell::new("Balance")
            .with_hspan(2)
            .style_spec("bcFC")]));

        table.set_format(format::FormatBuilder::new().padding(0, 5).build());

        table.printstd();
    }

    fn table_total(totals: &BTreeMap<String, Money>) {
        let mut table = Table::new();

        let line = totals
            .iter()
            .map(|(_, v)| Cell::new(&format!("{}", v)).style_spec("brFB"))
            .collect();

        table.add_row(Row::new(line));

        table.set_titles(Row::new(vec![Cell::new("Totals")
            .with_hspan(totals.len())
            .style_spec("bcFC")]));

        table.set_format(
            format::FormatBuilder::new()
                .separators(
                    &[format::LinePosition::Top],
                    format::LineSeparator::new('─', '┬', '┌', '┐'),
                )
                .padding(0, 5)
                .build(),
        );

        table.printstd();
    }
}
