use enum_dispatch::enum_dispatch;
use std::fs::File;

use crate::entity::date::Date;
use crate::entity::{entry::Entry, money::Currency, money::Money, transaction::Transaction};
use crate::exchange::Exchange;
use crate::CliResult;

#[enum_dispatch]
pub enum Line {
    Transaction,
    Entry,
}

impl Line {
    pub fn default(networth: bool) -> Line {
        if networth {
            Entry::default().into()
        } else {
            Transaction::default().into()
        }
    }

    pub fn build(values: Vec<String>, networth: bool) -> CliResult<Line> {
        if networth {
            Ok(Entry::build(values)?.into())
        } else {
            Ok(Transaction::build(values)?.into())
        }
    }
}

#[enum_dispatch(Line)]
pub trait Liner {
    fn headers(&self) -> Vec<&'static str>;
    fn account(&self) -> String;
    fn category(&self) -> String;
    fn amount(&self) -> Money;
    fn date(&self) -> Date;
    fn currency(&self) -> Currency;
    fn exchange(&self, to: Option<Currency>, exchange: &Exchange) -> CliResult<Line>;
    fn write(&self, wrt: &mut csv::Writer<File>) -> CliResult<()>;
}
