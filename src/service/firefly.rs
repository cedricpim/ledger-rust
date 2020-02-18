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

use crate::entity::line::{Line, Liner};

static BASE_PATH: &str = "https://demo.firefly-iii.org";

custom_error! { pub Error
    ReqwestError { source: reqwest::Error }       = @{ source },
    ApiError { source: firefly_iii::apis::Error } = @{ source },
    Value { source: std::num::ParseIntError }     = @{ source },

    DestinationAccountMissing = "The destination account for the transfer is missing",
}

pub struct Firefly {
    client: APIClient,
}

impl Firefly {
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
            match self
                .client
                .accounts_api()
                .list_account(page, None, None)
                .await
            {
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
    pub async fn create_account(
        &self,
        line: &Line,
        account_name: String,
        with_balance: bool,
        _type: account::Type,
    ) -> Result<String, Error> {
        let mut account = account::Account::new(account_name, _type);

        account.currency_code = Some(line.currency().code());

        if let account::Type::Asset = account._type {
            account.account_role = Some(account::AccountRole::DefaultAsset);
        };

        if with_balance {
            account.opening_balance = Some(line.amount().to_number());
            account.opening_balance_date = Some(line.date().format("%Y-%m-%d").to_string());
        };

        let response = self
            .client
            .accounts_api()
            .store_account(account)
            .await
            .map_err(Error::from)?;

        Ok(response.data.map(|v| v.id).unwrap_or_default())
    }

    #[tokio::main]
    pub async fn create_transaction(
        &self,
        line: &Line,
        other_line: Option<&Line>,
        balancesheet_id: i32,
        profit_loss_id: i32,
        transfer: bool,
    ) -> Result<String, Error> {
        let mut split = TransactionSplit::new(
            line.date().format("%Y-%m-%d").to_string(),
            line.amount().abs().to_number().to_string(),
            line.description(),
            None,
            None,
        );

        split.currency_code = Some(line.currency().code());
        split.category_name = Some(line.venue());
        split.tags = Some(vec![line.trip()]);
        split.notes = Some(line.quantity());

        if line.amount().positive() {
            split._type = Some(transaction_split::Type::Deposit);
            split.source_id = Some(profit_loss_id);
            split.destination_id = Some(balancesheet_id);
        } else {
            split._type = Some(transaction_split::Type::Withdrawal);
            split.source_id = Some(balancesheet_id);
            split.destination_id = Some(profit_loss_id);
        };

        if transfer {
            let destination_line = other_line.ok_or(Error::DestinationAccountMissing)?;

            split._type = Some(transaction_split::Type::Transfer);
            split.foreign_currency_code = Some(destination_line.currency().code());
            split.foreign_amount = Some(destination_line.amount().abs().to_number().to_string());
        };

        let transaction = Transaction::new(vec![split]);

        let response = self
            .client
            .transactions_api()
            .store_transaction(transaction)
            .await
            .map_err(Error::from)?;

        Ok(response.data.map(|v| v.id).unwrap_or_default())
    }

    pub fn type_for(&self, line: &Line, balancesheet: bool) -> account::Type {
        if balancesheet {
            account::Type::Asset
        } else if line.amount().positive() {
            account::Type::Revenue
        } else {
            account::Type::Expense
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
}
