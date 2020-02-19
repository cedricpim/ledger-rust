use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::service::firefly::Firefly;

pub struct AccountData {
    pub name: String,
    pub date: Date,
    pub currency: String,
    pub value: Option<Money>,
}

pub enum Account {
    BalanceSheet { data: AccountData },
    ProfitAndLoss { data: AccountData },
}

impl Account {
    pub fn doubleside(record: &Line, value: Option<Money>) -> (Self, Self) {
        (
            Account::new(&record, record.account(), None),
            Account::new(&record, record.category(), value),
        )
    }

    pub fn new(record: &Line, name: String, value: Option<Money>) -> Self {
        let data = AccountData {
            name,
            date: record.date(),
            currency: record.currency().code(),
            value,
        };

        if record.account() == data.name {
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
