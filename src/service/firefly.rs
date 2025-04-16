use anyhow::{anyhow, Result};
use firefly_iii::apis::configuration::Configuration;
use firefly_iii::apis::{about_api, accounts_api, currencies_api, transactions_api};
use firefly_iii::models::AccountRead;
use firefly_iii::models::AccountRoleProperty;
use firefly_iii::models::AccountStore;
use firefly_iii::models::CurrencyRead;
use firefly_iii::models::CurrencySingle;
use firefly_iii::models::MetaPagination;
use firefly_iii::models::ShortAccountTypeProperty;
use firefly_iii::models::TransactionRead;
use firefly_iii::models::TransactionSplitStore;
use firefly_iii::models::TransactionStore;
use firefly_iii::models::TransactionTypeFilter;
use firefly_iii::models::TransactionTypeProperty;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header;

use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::entity::sync::push;

static PROGRESS_BAR_FORMAT: &str = "{spinner:.green}▕{wide_bar:.cyan}▏{percent}% ({eta})";
static PROGRESS_BAR_CHARS: &str = "█▉▊▋▌▍▎▏  ";

pub struct Firefly {
    configuration: Configuration,
}

impl Firefly {
    pub fn new(base_path: &str, token: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        Self {
            configuration: Configuration {
                base_path: base_path.to_string(),
                user_agent: None,
                oauth_access_token: Some(token.to_string()),
                client: reqwest::Client::builder()
                    .default_headers(headers)
                    .connection_verbose(true)
                    .build()
                    .unwrap_or(reqwest::Client::new()),
                ..Default::default()
            },
        }
    }

    #[tokio::main]
    pub async fn user(&self) -> Result<String> {
        let response = about_api::get_current_user(&self.configuration).await?;

        Ok(response.data.id)
    }

    #[tokio::main]
    pub async fn default_currency(&self, code: String) -> Result<CurrencySingle> {
        Ok(currencies_api::default_currency(&self.configuration, &code).await?)
    }

    #[tokio::main]
    pub async fn enable_currency(&self, code: String) -> Result<CurrencySingle> {
        Ok(currencies_api::enable_currency(&self.configuration, &code).await?)
    }

    #[tokio::main]
    pub async fn currencies(&self) -> Result<Vec<CurrencyRead>> {
        let mut result: Vec<CurrencyRead> = Vec::new();

        let mut page = 0;

        loop {
            let response = currencies_api::list_currency(&self.configuration, Some(page)).await?;

            for currency in response.data {
                result.push(currency)
            }

            if let Some(val) = response.meta.pagination.and_then(Self::next_page) {
                page = val
            } else {
                break;
            }
        }

        Ok(result)
    }

    #[tokio::main]
    pub async fn transactions(&self, from: i32) -> Result<Vec<TransactionRead>> {
        let mut result: Vec<TransactionRead> = Vec::new();

        let (missing_entries, per_page) = self.missing_entries_per_page(from).await?;

        if missing_entries == 0 {
            return Ok(result);
        };

        let pb = ProgressBar::new(missing_entries as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(PROGRESS_BAR_FORMAT)
                .unwrap()
                .progress_chars(PROGRESS_BAR_CHARS),
        );

        let mut page = Some((missing_entries as f32 / per_page as f32).ceil() as i32);

        loop {
            let response = transactions_api::list_transaction(
                &self.configuration,
                page,
                None,
                None,
                Some(TransactionTypeFilter::All),
            )
            .await?;

            for transaction in response.data {
                let id = transaction.id.parse::<i32>().unwrap_or_default();

                if id > from {
                    pb.set_position((id - from) as u64);
                    result.push(transaction)
                }
            }

            if let Some(val) = response.meta.pagination.and_then(Self::previous_page) {
                page = Some(val);
            } else {
                break;
            }
        }

        Ok(result)
    }

    #[tokio::main]
    pub async fn accounts(&self) -> Result<Vec<AccountRead>> {
        let mut result: Vec<AccountRead> = Vec::new();

        let mut page = 0;

        loop {
            let response =
                accounts_api::list_account(&self.configuration, Some(page), None, None).await?;

            for account in response.data {
                result.push(account)
            }

            if let Some(val) = response.meta.pagination.and_then(Self::next_page) {
                page = val
            } else {
                break;
            }
        }

        Ok(result)
    }

    #[tokio::main]
    pub async fn get_opening_balance_transaction(&self, id: String) -> Result<String> {
        let mut response = accounts_api::list_transaction_by_account(
            &self.configuration,
            &id,
            None,
            Some(1),
            None,
            None,
            Some(TransactionTypeFilter::OpeningBalance),
        )
        .await?;

        response.data.pop().map(|v| v.id).ok_or(anyhow!(
            "Account {} is missing an opening balance transaction",
            id
        ))
    }

    #[tokio::main]
    pub async fn create_account(&self, data: &push::AccountData) -> Result<String> {
        let mut account = AccountStore::new(data.name.to_string(), data._type);

        account.currency_code = data.currency.clone();
        account.include_net_worth = Some(data.networth);
        account.opening_balance = data.value.map(|v| v.to_storage());
        account.opening_balance_date = data.date.map(|v| v.to_string());

        if let ShortAccountTypeProperty::Asset = account._type {
            account.account_role = Some(AccountRoleProperty::DefaultAsset);
        };

        let response = accounts_api::store_account(&self.configuration, account).await?;

        Ok(response.data.id)
    }

    #[tokio::main]
    pub async fn create_transfer(&self, transfer: push::Transfer, user: i32) -> Result<String> {
        if transfer.value().zero() {
            return Ok(String::new());
        }

        let mut split = Firefly::build_split(
            TransactionTypeProperty::Transfer,
            transfer.line,
            transfer.value(),
            transfer.ids,
        );

        split.foreign_currency_code = Some(transfer.other_line.currency().code());
        split.foreign_amount = Some(transfer.other_line.amount().abs().to_number().to_string());

        self.post_transaction(split, user).await
    }

    #[tokio::main]
    pub async fn create_transaction(
        &self,
        transaction: push::Transaction,
        user: i32,
    ) -> Result<String> {
        if transaction.value().zero() {
            return Ok(String::new());
        }

        let split = if transaction.value().positive() {
            Firefly::build_split(
                TransactionTypeProperty::Deposit,
                transaction.line,
                transaction.value(),
                transaction.ids,
            )
        } else {
            Firefly::build_split(
                TransactionTypeProperty::Withdrawal,
                transaction.line,
                transaction.value(),
                transaction.ids,
            )
        };

        self.post_transaction(split, user).await
    }

    async fn post_transaction(&self, split: TransactionSplitStore, _user: i32) -> Result<String> {
        let transaction = TransactionStore::new(vec![split]);

        let response =
            transactions_api::store_transaction(&self.configuration, transaction).await?;

        Ok(response.data.id)
    }

    fn build_split(
        _type: TransactionTypeProperty,
        line: &Line,
        amount: Money,
        ids: (String, String),
    ) -> TransactionSplitStore {
        let mut split = TransactionSplitStore::new(
            _type,
            line.date().to_string(),
            amount.abs().to_number().to_string(),
            line.description(),
        );

        split.source_id = Some(ids.0);
        split.destination_id = Some(ids.1);
        split.currency_code = Some(line.currency().code());
        split.currency_code = Some(line.currency().code());
        split.category_name = Some(line.venue());
        split.tags = Some(vec![line.trip()]);
        split.notes = Some(line.quantity());

        split
    }

    async fn missing_entries_per_page(&self, from: i32) -> Result<(i32, i32)> {
        let response = transactions_api::list_transaction(
            &self.configuration,
            None,
            None,
            None,
            Some(TransactionTypeFilter::All),
        )
        .await?;

        let per_page = response
            .meta
            .pagination
            .map_or(0, |v| v.per_page.unwrap_or_default());

        match response.data.first() {
            None => Ok((0, per_page)),
            Some(transaction) => {
                let id = transaction.id.parse::<i32>()?;
                Ok((id - from, per_page))
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
