use custom_error::custom_error;
use firefly_iii::apis::client::APIClient;
use firefly_iii::apis::configuration::Configuration;
use firefly_iii::models::account;
use firefly_iii::models::account_read::AccountRead;
use firefly_iii::models::currency_read::CurrencyRead;
use firefly_iii::models::currency_single::CurrencySingle;
use firefly_iii::models::meta_pagination::MetaPagination;
use firefly_iii::models::transaction::Transaction;
use firefly_iii::models::transaction_split;
use firefly_iii::models::transaction_split::TransactionSplit;
use firefly_iii::models::transaction_type_filter::TransactionTypeFilter;

use crate::entity::line::{Line, Liner};
use crate::entity::money::Money;
use crate::entity::sync;

static BASE_PATH: &str = "http://localhost";

custom_error! { pub Error
    ReqwestError { source: reqwest::Error }       = @{ source },
    ApiError { source: firefly_iii::apis::Error } = @{ source },

    DestinationAccountMissing     = "The destination account for the transfer is missing",
    MissingResponseData           = "The data is missing from response",
    MissingExpectedOpeningBalance = "The account is missing an opening balance transaction",
}

pub struct Firefly {
    client: APIClient,
}

impl Firefly {
    pub fn type_for(account: &sync::Account) -> account::Type {
        match account {
            sync::Account::BalanceSheet { data } => match data.value {
                Some(val) if val.negative() => account::Type::Liability,
                _ => account::Type::Asset,
            },
            sync::Account::ProfitAndLoss { data } => match data.value {
                Some(val) if val.negative() => account::Type::Expense,
                _ => account::Type::Revenue,
            },
        }
    }

    pub fn new(token: String) -> Self {
        Self {
            client: APIClient::new(Configuration {
                base_path: BASE_PATH.to_string(),
                user_agent: None,
                oauth_access_token: Some(token),
                ..Default::default()
            }),
        }
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

        let mut page = None;

        loop {
            match self.client.currencies_api().list_currency(page).await {
                Err(e) => return Err(Error::from(e)),
                Ok(val) => {
                    for currency in val.data {
                        result.push(currency)
                    }

                    if let Some(val) = val.meta.pagination.and_then(Self::next_page) {
                        page = Some(val)
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

        let mut page = None;

        loop {
            let response = self
                .client
                .accounts_api()
                .list_account(page, None, None)
                .await;

            match response {
                Err(e) => return Err(Error::from(e)),
                Ok(val) => {
                    for account in val.data {
                        result.push(account)
                    }

                    if let Some(val) = val.meta.pagination.and_then(Self::next_page) {
                        page = Some(val)
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
    pub async fn create_account(&self, info: sync::Account) -> Result<String, Error> {
        let data = info.data();

        let mut account = account::Account::new(data.name.to_string(), Firefly::type_for(&info));

        account.currency_code = Some(data.currency.to_string());
        account.include_net_worth = Some(data.networth);

        if let account::Type::Asset = account._type {
            account.account_role = Some(account::AccountRole::DefaultAsset);
        };

        if let sync::Account::BalanceSheet { data } = info {
            if let Some(val) = data.value {
                account.opening_balance = Some(val.to_number());
                account.opening_balance_date = Some(data.date.format("%Y-%m-%d").to_string());
            };
        }

        let response = self
            .client
            .accounts_api()
            .store_account(account)
            .await?;

        response.data.map(|v| v.id).ok_or(Error::MissingResponseData)
    }

    #[tokio::main]
    pub async fn create_transaction(
        &self,
        line: &Line,
        other_line: Option<&Line>,
        balancesheet_id: i32,
        profit_loss_id: i32,
        amount: Money,
        transfer: bool,
    ) -> Result<String, Error> {
        let mut split = TransactionSplit::new(
            line.date().format("%Y-%m-%d").to_string(),
            amount.abs().to_number().to_string(),
            line.description(),
            None,
            None,
        );

        split.currency_code = Some(line.currency().code());
        split.category_name = Some(line.venue());
        split.tags = Some(vec![line.trip()]);
        split.notes = Some(line.quantity());

        if amount.positive() {
            split._type = Some(transaction_split::Type::Deposit);
            split.source_id = Some(profit_loss_id);
            split.destination_id = Some(balancesheet_id);
        } else if amount.negative() {
            split._type = Some(transaction_split::Type::Withdrawal);
            split.source_id = Some(balancesheet_id);
            split.destination_id = Some(profit_loss_id);
        } else {
            return Ok(String::new());
        };

        if transfer {
            let destination_line = other_line.ok_or(Error::DestinationAccountMissing)?;

            split._type = Some(transaction_split::Type::Transfer);
            split.foreign_currency_code = Some(destination_line.currency().code());
            split.foreign_amount = Some(destination_line.amount().abs().to_number().to_string());
        };

        println!("{:?}", split);

        let transaction = Transaction::new(vec![split]);

        let response = self
            .client
            .transactions_api()
            .store_transaction(transaction)
            .await?;

        Ok(response.data.map(|v| v.id).unwrap_or_default())
    }

    fn next_page(pagination: MetaPagination) -> Option<i32> {
        let current_page = pagination.current_page.unwrap_or(1);

        if pagination.total_pages.unwrap_or(1) > current_page {
            Some(current_page + 1)
        } else {
            None
        }
    }
}
