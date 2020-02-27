use firefly_iii::models::account::Type;

use crate::cmd::sync::Sync;
use crate::config::FireflyOptions;
use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::error::CliError;
use crate::filter::Filter;
use crate::service::firefly::Firefly;
use crate::CliResult;

pub struct AccountData {
    pub id: Option<i32>,
    pub name: String,
    pub date: Option<Date>,
    pub value: Option<Money>,
    pub currency: Option<String>,
    pub networth: bool,
    pub _type: Type,
}

impl AccountData {
    fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            date: None,
            value: None,
            currency: None,
            networth: false,
            _type: Type::Asset,
        }
    }

    fn asset(line: &Line, filter: &Filter) -> Self {
        let mut data = Self::new(line.account());

        data.currency = Some(line.currency().code());
        data.networth = filter.accountable(&line.account());
        data._type = Type::Asset;

        data
    }

    pub fn key(&self) -> (String, String) {
        (self.name.to_string(), self._type.to_string())
    }
}

pub enum Account {
    Balance(AccountData),
    DoubleEntry(AccountData, AccountData),
}

impl Account {
    fn balance(line: &Line, value: Money, filter: &Filter, sync: &mut Sync) -> CliResult<Self> {
        let mut account = AccountData::asset(&line, &filter);

        account.date = Some(line.date());
        account.value = Some(value);
        account.id = Some(sync.account(&account)?);

        Ok(Self::Balance(account))
    }

    fn transaction(line: &Line, value: Money, filter: &Filter, sync: &mut Sync) -> CliResult<Self> {
        let mut asset = AccountData::asset(&line, &filter);
        asset.id = Some(sync.account(&asset)?);

        if value.negative() {
            let mut expense = AccountData::new(line.category());
            expense._type = Type::Expense;
            expense.id = Some(sync.account(&expense)?);

            Ok(Self::DoubleEntry(asset, expense))
        } else {
            let mut revenue = AccountData::new(line.category());
            revenue._type = Type::Revenue;
            revenue.id = Some(sync.account(&revenue)?);

            Ok(Self::DoubleEntry(revenue, asset))
        }
    }

    fn transfer(
        line: &Line,
        other_line: &Line,
        filter: &Filter,
        sync: &mut Sync,
    ) -> CliResult<Self> {
        let mut one = AccountData::asset(&line, &filter);
        one.id = Some(sync.account(&one)?);

        let mut other = AccountData::asset(&other_line, &filter);
        other.id = Some(sync.account(&other)?);

        if line.amount().negative() {
            Ok(Self::DoubleEntry(one, other))
        } else {
            Ok(Self::DoubleEntry(other, one))
        }
    }

    fn id(&self) -> CliResult<i32> {
        match &*self {
            Self::Balance(account) => account.id.ok_or(CliError::MissingAccountId),
            Self::DoubleEntry(_, _) => Err(CliError::MissingAccountId),
        }
    }

    fn ids(&self) -> CliResult<(i32, i32)> {
        match &*self {
            Self::Balance(_) => Err(CliError::MissingAccountId),
            Self::DoubleEntry(one, other) => Ok((
                one.id.ok_or(CliError::MissingAccountId)?,
                other.id.ok_or(CliError::MissingAccountId)?,
            )),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Transaction<'a> {
    pub ids: (i32, i32),
    pub line: &'a Line,
    pub value: Money,
}

impl<'a> Transaction<'a> {
    pub fn new(line: &'a Line, value: Money, filter: &Filter, sync: &mut Sync) -> CliResult<Self> {
        let accounts = Account::transaction(&line, value, &filter, sync)?;

        Ok(Self {
            ids: accounts.ids()?,
            line,
            value,
        })
    }

    pub fn value(&self) -> Money {
        self.value
    }
}

#[derive(Copy, Clone)]
pub struct Transfer<'a> {
    pub ids: (i32, i32),
    pub line: &'a Line,
    pub other_line: &'a Line,
}

impl<'a> Transfer<'a> {
    pub fn new(
        line: &'a Line,
        other_line: &'a Line,
        filter: &Filter,
        sync: &mut Sync,
    ) -> CliResult<Self> {
        let accounts = Account::transfer(&line, &other_line, &filter, sync)?;

        Ok(Self {
            ids: accounts.ids()?,
            line,
            other_line,
        })
    }

    pub fn value(&self) -> Money {
        self.line.amount()
    }
}

pub struct Ledger<'a> {
    user: i32,
    filter: &'a Filter,
    firefly: &'a Firefly,
    options: FireflyOptions,
    pub previous: Option<Line>,
}

impl<'a> Ledger<'a> {
    pub fn new(
        user: i32,
        filter: &'a Filter,
        firefly: &'a Firefly,
        options: FireflyOptions,
    ) -> Self {
        Self {
            user,
            filter,
            options,
            firefly,
            previous: None,
        }
    }

    fn process_transfer(
        &mut self,
        record: &Line,
        sync: &mut Sync,
    ) -> CliResult<(String, Vec<Line>)> {
        let (mut id, mut lines) = (String::new(), Vec::<Line>::new());

        match &self.previous {
            Some(val) => {
                let transfer = Transfer::new(&val, &record, &self.filter, sync)?;

                id = self.firefly().create_transfer(transfer, self.user)?;

                lines = vec![self.previous.take().unwrap(), record.clone()];
            }
            None => self.previous = Some(record.clone()),
        };

        Ok((id, lines))
    }
}

#[derive(Copy, Clone)]
pub struct Networth<'a> {
    user: i32,
    filter: &'a Filter,
    firefly: &'a Firefly,
    previous_amount: Option<Money>,
}

impl<'a> Networth<'a> {
    pub fn new(user: i32, filter: &'a Filter, firefly: &'a Firefly) -> Self {
        Self {
            user,
            filter,
            firefly,
            previous_amount: None,
        }
    }
}

pub trait Syncable<'a> {
    fn process(&mut self, record: &Line, sync: &mut Sync) -> CliResult<(String, Vec<Line>)>;

    fn opening_balance(&self, record: &Line) -> bool;

    fn balance(&self, record: &Line) -> Money;

    fn value(&self, record: &Line) -> Money;

    fn filter(&self) -> &'a Filter;

    fn user(&self) -> i32;

    fn firefly(&self) -> &'a Firefly;

    fn previous(&self) -> Option<&Line>;

    fn process_transaction(
        &mut self,
        record: &Line,
        sync: &mut Sync,
    ) -> CliResult<(String, Vec<Line>)> {
        let id = if self.opening_balance(&record) {
            let account = Account::balance(&record, self.balance(&record), self.filter(), sync)?;

            self.firefly()
                .get_opening_balance_transaction(account.id()?)?
        } else {
            let transaction = Transaction::new(&record, self.value(&record), self.filter(), sync)?;

            self.firefly()
                .create_transaction(transaction, self.user())?
        };

        Ok((id, vec![record.clone()]))
    }
}

impl<'a> Syncable<'a> for Ledger<'a> {
    fn process(&mut self, record: &Line, sync: &mut Sync) -> CliResult<(String, Vec<Line>)> {
        if record.syncable() {
            if record.category() == self.options.transfer {
                self.process_transfer(&record, sync)
            } else {
                self.process_transaction(&record, sync)
            }
        } else {
            Ok(record.synced())
        }
    }

    fn opening_balance(&self, record: &Line) -> bool {
        record.category() == self.options.opening_balance
    }

    fn balance(&self, record: &Line) -> Money {
        record.amount()
    }

    fn value(&self, record: &Line) -> Money {
        record.amount()
    }

    fn filter(&self) -> &'a Filter {
        self.filter
    }

    fn user(&self) -> i32 {
        self.user
    }

    fn firefly(&self) -> &'a Firefly {
        self.firefly
    }

    fn previous(&self) -> Option<&Line> {
        self.previous.as_ref()
    }
}

impl<'a> Syncable<'a> for Networth<'a> {
    fn process(&mut self, record: &Line, sync: &mut Sync) -> CliResult<(String, Vec<Line>)> {
        let result = if record.syncable() {
            self.process_transaction(&record, sync)?
        } else {
            record.synced()
        };

        self.previous_amount = Some(record.investment());

        Ok(result)
    }

    fn opening_balance(&self, _record: &Line) -> bool {
        self.previous_amount.is_none()
    }

    fn balance(&self, record: &Line) -> Money {
        record.investment()
    }

    fn value(&self, record: &Line) -> Money {
        record.investment() - self.previous_amount.unwrap_or_default()
    }

    fn filter(&self) -> &'a Filter {
        self.filter
    }

    fn user(&self) -> i32 {
        self.user
    }

    fn firefly(&self) -> &'a Firefly {
        self.firefly
    }

    fn previous(&self) -> Option<&Line> {
        None
    }
}
