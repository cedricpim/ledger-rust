use anyhow::anyhow;
use firefly_iii::models::ShortAccountTypeProperty;
use indicatif::{ProgressBar, ProgressStyle};

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use crate::config::Config;
use crate::config::FireflyOptions;
use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::filter::Filter;
use crate::resource::Resource;
use crate::service::firefly::Firefly;
use crate::Mode;

static PROGRESS_BAR_FORMAT: &str = "{spinner:.green}▕{wide_bar:.cyan}▏{percent}% ({eta})";
static PROGRESS_BAR_CHARS: &str = "█▉▊▋▌▍▎▏  ";

pub struct Push {
    user: i32,
    firefly: Firefly,
    options: FireflyOptions,
    currencies: HashSet<String>,
    accounts: HashMap<(String, String), String>,
}

impl Push {
    fn process<F>(
        config: &Config,
        mode: Mode,
        pb: &ProgressBar,
        action: &mut F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(&mut Line, &mut Option<anyhow::Error>) -> anyhow::Result<(String, Vec<Line>)>,
    {
        let mut resource = Resource::new(config, mode)?;

        let mut error: Option<anyhow::Error> = None;

        resource.rewrite(&mut |record| {
            if record.pushable() {
                pb.inc(1);
            };

            let (id, mut lines) = action(record, &mut error)?;

            lines
                .iter_mut()
                .for_each(|line| line.set_id(id.to_string()));

            Ok(lines)
        })?;

        error.map_or(Ok(()), Err)
    }

    pub fn new(options: &FireflyOptions, config: &Config) -> anyhow::Result<Self> {
        let client = Firefly::new(&options.base_path, &options.token);

        Ok(Self {
            user: client.user()?.parse::<i32>()?,
            firefly: client,
            options: FireflyOptions::build(options, config),
            currencies: HashSet::new(),
            accounts: HashMap::new(),
        })
    }

    pub fn perform(&mut self, config: Config) -> anyhow::Result<()> {
        let pb = ProgressBar::new(config.total_pushable_lines()? as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(PROGRESS_BAR_FORMAT)
                .unwrap()
                .progress_chars(PROGRESS_BAR_CHARS),
        );

        self.load()?;

        let filter = Filter::push(&config);
        let client = Firefly::new(&self.options.base_path, &self.options.token);

        let mut ledger = Ledger::new(self.user, &filter, &client, self.options.clone());
        self.push(&config, Mode::Ledger, &mut ledger, &pb)?;

        let mut networth = Networth::new(self.user, &filter, &client);
        self.push(&config, Mode::Networth, &mut networth, &pb)?;

        Ok(())
    }

    pub fn account(&mut self, account: &AccountData) -> anyhow::Result<String> {
        match self.accounts.entry(account.key()) {
            Entry::Occupied(v) => Ok(v.get().to_string()),
            Entry::Vacant(v) => {
                let id = self.firefly.create_account(account)?;

                v.insert(id.to_string());

                Ok(id)
            }
        }
    }

    fn push<'a, T>(
        &mut self,
        config: &Config,
        mode: Mode,
        entity: &'a mut T,
        pb: &ProgressBar,
    ) -> anyhow::Result<()>
    where
        T: Pushable<'a>,
    {
        Self::process(config, mode, pb, &mut |record, error| match error {
            None => {
                let result = self
                    .process_currency(record)
                    .and(entity.process(record, self));

                let handle_error = |e: anyhow::Error| -> anyhow::Result<(String, Vec<Line>)> {
                    *error = Some(e);
                    Ok(entity.previous().map_or_else(
                        || record.pushed(),
                        |v| (record.id(), vec![v.clone(), record.clone()]),
                    ))
                };

                result.or_else(handle_error)
            }
            Some(_) => Ok(record.pushed()),
        })
    }

    fn process_currency(&mut self, record: &Line) -> anyhow::Result<()> {
        if !self.currencies.contains(&record.currency().code()) {
            self.firefly.enable_currency(record.currency().code())?;
            self.currencies.insert(record.currency().code());
        }

        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        self.firefly
            .default_currency(self.options.currency.to_string())?;

        for account in self.firefly.accounts()? {
            let info = (
                account.attributes.name.to_string(),
                format!("{:?}", account.attributes._type),
            );

            self.accounts.entry(info).or_insert_with(|| account.id);
        }

        for currency in self.firefly.currencies()? {
            if currency.attributes.enabled.unwrap_or_default() {
                self.currencies.insert(currency.attributes.code);
            }
        }

        Ok(())
    }
}

pub struct AccountData {
    pub id: Option<String>,
    pub name: String,
    pub date: Option<Date>,
    pub value: Option<Money>,
    pub currency: Option<String>,
    pub networth: bool,
    pub _type: ShortAccountTypeProperty,
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
            _type: ShortAccountTypeProperty::Asset,
        }
    }

    fn asset(line: &Line, filter: &Filter) -> Self {
        let mut data = Self::new(line.account());

        data.currency = Some(line.currency().code());
        data.networth = filter.accountable(&line.account());
        data._type = ShortAccountTypeProperty::Asset;

        data
    }

    pub fn key(&self) -> (String, String) {
        (self.name.to_string(), format!("{:?}", self._type))
    }
}

pub enum Account {
    Balance(AccountData),
    DoubleEntry(AccountData, AccountData),
}

impl Account {
    fn balance(
        line: &Line,
        value: Money,
        filter: &Filter,
        push: &mut Push,
    ) -> anyhow::Result<Self> {
        let mut account = AccountData::asset(line, filter);

        account.date = Some(line.date());
        account.value = Some(value);
        account.id = Some(push.account(&account)?);

        Ok(Self::Balance(account))
    }

    fn transaction(
        line: &Line,
        value: Money,
        filter: &Filter,
        push: &mut Push,
    ) -> anyhow::Result<Self> {
        let mut asset = AccountData::asset(line, filter);
        asset.id = Some(push.account(&asset)?);

        let mut data = AccountData::new(line.category());
        if value.negative() {
            data._type = ShortAccountTypeProperty::Expense;
            data.id = Some(push.account(&data)?);

            Ok(Self::DoubleEntry(asset, data))
        } else {
            data._type = ShortAccountTypeProperty::Revenue;
            data.id = Some(push.account(&data)?);

            Ok(Self::DoubleEntry(data, asset))
        }
    }

    fn transfer(
        line: &Line,
        other_line: &Line,
        filter: &Filter,
        push: &mut Push,
    ) -> anyhow::Result<Self> {
        let mut one = AccountData::asset(line, filter);
        one.id = Some(push.account(&one)?);

        let mut other = AccountData::asset(other_line, filter);
        other.id = Some(push.account(&other)?);

        if line.amount().negative() {
            Ok(Self::DoubleEntry(one, other))
        } else {
            Ok(Self::DoubleEntry(other, one))
        }
    }

    fn id(&self) -> anyhow::Result<String> {
        match self {
            Self::Balance(account) => account
                .id
                .clone()
                .ok_or(anyhow!("Id of the account is missing")),
            Self::DoubleEntry(_, _) => Err(anyhow!("Id of the account is missing")),
        }
    }

    fn ids(&self) -> anyhow::Result<(String, String)> {
        match self {
            Self::Balance(_) => Err(anyhow!("Id of the account is missing")),
            Self::DoubleEntry(one, other) => Ok((
                one.id
                    .clone()
                    .ok_or(anyhow!("Id of the account is missing"))?,
                other
                    .id
                    .clone()
                    .ok_or(anyhow!("Id of the account is missing"))?,
            )),
        }
    }
}

#[derive(Clone)]
pub struct Transaction<'a> {
    pub ids: (String, String),
    pub line: &'a Line,
    pub value: Money,
}

impl<'a> Transaction<'a> {
    pub fn new(
        line: &'a Line,
        value: Money,
        filter: &Filter,
        push: &mut Push,
    ) -> anyhow::Result<Self> {
        let accounts = Account::transaction(line, value, filter, push)?;

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

#[derive(Clone)]
pub struct Transfer<'a> {
    pub ids: (String, String),
    pub line: &'a Line,
    pub other_line: &'a Line,
}

impl<'a> Transfer<'a> {
    pub fn new(
        line: &'a Line,
        other_line: &'a Line,
        filter: &Filter,
        push: &mut Push,
    ) -> anyhow::Result<Self> {
        let accounts = Account::transfer(line, other_line, filter, push)?;

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
        push: &mut Push,
    ) -> anyhow::Result<(String, Vec<Line>)> {
        let (mut id, mut lines) = (String::new(), Vec::<Line>::new());

        match &self.previous {
            Some(val) => {
                let transfer = Transfer::new(val, record, self.filter, push)?;

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

pub trait Pushable<'a> {
    fn process(&mut self, record: &Line, push: &mut Push) -> anyhow::Result<(String, Vec<Line>)>;

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
        push: &mut Push,
    ) -> anyhow::Result<(String, Vec<Line>)> {
        let id = if self.opening_balance(record) {
            let account = Account::balance(record, self.balance(record), self.filter(), push)?;

            self.firefly()
                .get_opening_balance_transaction(account.id()?)?
        } else {
            let transaction = Transaction::new(record, self.value(record), self.filter(), push)?;

            self.firefly()
                .create_transaction(transaction, self.user())?
        };

        Ok((id, vec![record.clone()]))
    }
}

impl<'a> Pushable<'a> for Ledger<'a> {
    fn process(&mut self, record: &Line, push: &mut Push) -> anyhow::Result<(String, Vec<Line>)> {
        if record.pushable() {
            if record.category() == self.options.transfer {
                self.process_transfer(record, push)
            } else {
                self.process_transaction(record, push)
            }
        } else {
            Ok(record.pushed())
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

impl<'a> Pushable<'a> for Networth<'a> {
    fn process(&mut self, record: &Line, push: &mut Push) -> anyhow::Result<(String, Vec<Line>)> {
        let result = if record.pushable() {
            self.process_transaction(record, push)?
        } else {
            record.pushed()
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
