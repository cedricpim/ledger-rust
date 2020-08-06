use custom_error::custom_error;
use firefly_iii::apis::client::APIClient;
use firefly_iii::apis::configuration::Configuration;
use firefly_iii::models::account;
use firefly_iii::models::account_read::AccountRead;
use firefly_iii::models::currency_read::CurrencyRead;
use firefly_iii::models::currency_single::CurrencySingle;
use firefly_iii::models::meta_pagination::MetaPagination;
use firefly_iii::models::transaction::Transaction;
use firefly_iii::models::transaction_read::TransactionRead;
use firefly_iii::models::transaction_split;
use firefly_iii::models::transaction_split::TransactionSplit;
use firefly_iii::models::transaction_type_filter::TransactionTypeFilter;
use indicatif::{ProgressBar, ProgressStyle};

use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::entity::push;

custom_error! { pub Error
    ReqwestError { source: reqwest::Error }       = @{ source },
    ApiError { source: firefly_iii::apis::Error } = @{ source },
    Value { source: std::num::ParseIntError }     = @{ source },

    MissingExpectedOpeningBalance = "The account is missing an opening balance transaction",
}

static PROGRESS_BAR_FORMAT: &str = "{spinner:.green}▕{wide_bar:.cyan}▏{percent}% ({eta})";
static PROGRESS_BAR_CHARS: &str = "█▉▊▋▌▍▎▏  ";

pub struct Firefly {
    client: APIClient,
}

impl Firefly {
    pub fn new(base_path: &str, token: &str) -> Self {
        Self {
            client: APIClient::new(Configuration {
                base_path: base_path.to_string(),
                user_agent: None,
                oauth_access_token: Some(token.to_string()),
                ..Default::default()
            }),
        }
    }

    #[tokio::main]
    pub async fn user(&self) -> Result<String, Error> {
        let response = self.client.about_api().get_current_user().await?;

        Ok(response.data.id)
    }

    #[tokio::main]
    pub async fn default_currency(&self, code: String) -> Result<CurrencySingle, Error> {
        self.client
            .currencies_api()
            .default_currency(&code)
            .await
            .map_err(Error::from)
    }

    #[tokio::main]
    pub async fn enable_currency(&self, code: String) -> Result<CurrencySingle, Error> {
        self.client
            .currencies_api()
            .enable_currency(&code)
            .await
            .map_err(Error::from)
    }

    #[tokio::main]
    pub async fn currencies(&self) -> Result<Vec<CurrencyRead>, Error> {
        let mut result: Vec<CurrencyRead> = Vec::new();

        let mut page = 0;

        loop {
            let response = self.client.currencies_api().list_currency(Some(page)).await;

            match response {
                Err(e) => return Err(Error::from(e)),
                Ok(val) => {
                    for currency in val.data {
                        result.push(currency)
                    }

                    if let Some(val) = val.meta.pagination.and_then(Self::next_page) {
                        page = val
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    #[tokio::main]
    pub async fn transactions(&self, from: i32) -> Result<Vec<TransactionRead>, Error> {
        let mut result: Vec<TransactionRead> = Vec::new();

        let (missing_entries, per_page) = self.missing_entries_per_page(from).await?;

        if missing_entries == 0 {
            return Ok(result);
        };

        let pb = ProgressBar::new(missing_entries as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(PROGRESS_BAR_FORMAT)
                .progress_chars(PROGRESS_BAR_CHARS),
        );

        let mut page = Some((missing_entries as f32 / per_page as f32).ceil() as i32);

        loop {
            let response = self
                .client
                .transactions_api()
                .list_transaction(page, None, None, Some(TransactionTypeFilter::All))
                .await;

            match response {
                Err(e) => return Err(Error::from(e)),
                Ok(val) => {
                    for transaction in val.data {
                        let id = transaction.id.parse::<i32>().unwrap_or_default();

                        if id > from {
                            pb.set_position((id - from) as u64);
                            result.push(transaction)
                        }
                    }

                    if let Some(val) = val.meta.pagination.and_then(Self::previous_page) {
                        page = Some(val);
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    #[tokio::main]
    pub async fn accounts(&self) -> Result<Vec<AccountRead>, Error> {
        let mut result: Vec<AccountRead> = Vec::new();

        let mut page = 0;

        loop {
            let response = self
                .client
                .accounts_api()
                .list_account(Some(page), None, None)
                .await;

            match response {
                Err(e) => return Err(Error::from(e)),
                Ok(val) => {
                    for account in val.data {
                        result.push(account)
                    }

                    if let Some(val) = val.meta.pagination.and_then(Self::next_page) {
                        page = val
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    #[tokio::main]
    pub async fn get_opening_balance_transaction(&self, id: i32) -> Result<String, Error> {
        let mut response = self
            .client
            .accounts_api()
            .list_transaction_by_account(
                id,
                None,
                Some(1),
                None,
                None,
                Some(TransactionTypeFilter::OpeningBalance),
            )
            .await?;

        response
            .data
            .pop()
            .map(|v| v.id)
            .ok_or(Error::MissingExpectedOpeningBalance)
    }

    #[tokio::main]
    pub async fn create_account(&self, data: &push::AccountData) -> Result<String, Error> {
        let mut account = account::Account::new(data.name.to_string(), data._type.clone());

        account.currency_code = data.currency.clone();
        account.include_net_worth = Some(data.networth);
        account.opening_balance = data.value.map(|v| v.to_storage());
        account.opening_balance_date = data.date.map(|v| v.format("%Y-%m-%d").to_string());

        if let account::Type::Asset = account._type {
            account.account_role = Some(account::AccountRole::DefaultAsset);
        };

        let response = self.client.accounts_api().store_account(account).await?;

        Ok(response.data.id)
    }

    #[tokio::main]
    pub async fn create_transfer(
        &self,
        transfer: push::Transfer,
        user: i32,
    ) -> Result<String, Error> {
        if transfer.value().zero() {
            return Ok(String::new());
        }

        let mut split = Firefly::build_split(&transfer.line, transfer.value(), transfer.ids);

        split._type = Some(transaction_split::Type::Transfer);
        split.foreign_currency_code = Some(transfer.other_line.currency().code());
        split.foreign_amount = Some(transfer.other_line.amount().abs().to_number().to_string());

        self.post_transaction(split, user).await
    }

    #[tokio::main]
    pub async fn create_transaction(
        &self,
        transaction: push::Transaction,
        user: i32,
    ) -> Result<String, Error> {
        if transaction.value().zero() {
            return Ok(String::new());
        }

        let mut split =
            Firefly::build_split(&transaction.line, transaction.value(), transaction.ids);

        if transaction.value().positive() {
            split._type = Some(transaction_split::Type::Deposit);
        } else {
            split._type = Some(transaction_split::Type::Withdrawal);
        }

        self.post_transaction(split, user).await
    }

    async fn post_transaction(&self, split: TransactionSplit, user: i32) -> Result<String, Error> {
        let mut transaction = Transaction::new(vec![split]);

        transaction.user = Some(user);

        let response = self
            .client
            .transactions_api()
            .store_transaction(transaction)
            .await?;

        Ok(response.data.id)
    }

    fn build_split(line: &Line, amount: Money, ids: (i32, i32)) -> TransactionSplit {
        let mut split = TransactionSplit::new(
            line.date().format("%Y-%m-%d").to_string(),
            amount.abs().to_number().to_string(),
            line.description(),
            Some(ids.0),
            Some(ids.1),
        );

        split.currency_code = Some(line.currency().code());
        split.category_name = Some(line.venue());
        split.tags = Some(vec![line.trip()]);
        split.notes = Some(line.quantity());

        split
    }

    async fn missing_entries_per_page(&self, from: i32) -> Result<(i32, i32), Error> {
        match self
            .client
            .transactions_api()
            .list_transaction(None, None, None, Some(TransactionTypeFilter::All))
            .await
        {
            Err(e) => Err(Error::from(e)),
            Ok(val) => {
                let per_page = val
                    .meta
                    .pagination
                    .map_or(0, |v| v.per_page.unwrap_or_default());

                match val.data.first() {
                    None => Ok((0, per_page)),
                    Some(transaction) => match transaction.id.parse::<i32>() {
                        Err(e) => Err(Error::from(e)),
                        Ok(id) => Ok((id - from, per_page)),
                    },
                }
            }
        }
    }

    fn next_page(pagination: MetaPagination) -> Option<i32> {
        let current_page = pagination.current_page.unwrap_or(1);

        if pagination.total_pages.unwrap_or(1) > current_page {
            Some(current_page + 1)
        } else {
            None
        }
    }

    fn previous_page(pagination: MetaPagination) -> Option<i32> {
        let current_page = pagination.current_page.unwrap_or(1);

        if current_page != 1 {
            Some(current_page - 1)
        } else {
            None
        }
    }
}
