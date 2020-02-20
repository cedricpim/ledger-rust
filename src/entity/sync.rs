use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::service::firefly::Firefly;
use crate::filter::Filter;

pub struct AccountData {
    pub name: String,
    pub date: Date,
    pub currency: String,
    pub value: Option<Money>,
    pub networth: bool,
}

pub enum Account {
    BalanceSheet { data: AccountData },
    ProfitAndLoss { data: AccountData },
}

impl Account {
    pub fn doubleside(record: &Line, value: Option<Money>, filter: &Filter) -> (Self, Self) {
        (
            Account::new(&record, record.account(), None, filter),
            Account::new(&record, record.category(), value, filter),
        )
    }

    pub fn new(record: &Line, name: String, value: Option<Money>, filter: &Filter) -> Self {
        let mut data = AccountData {
            name,
            date: record.date(),
            currency: record.currency().code(),
            value,
            networth: true,
        };

        if record.account() == data.name {
            data.networth = !filter.ignore_account(&data.name);

            Account::BalanceSheet { data }
        } else {
            Account::ProfitAndLoss { data }
        }
    }

    pub fn data(&self) -> &AccountData {
        match self {
            Account::BalanceSheet { data } => data,
            Account::ProfitAndLoss { data } => data,
        }
    }

    pub fn key(&self) -> (String, String) {
        (
            self.data().name.to_string(),
            Firefly::type_for(self).to_string(),
        )
    }
}
