use super::account::NordigenAccount;
use super::asset::GenericAsset;
use super::cryptoprice::get_token_price;
use super::error::ApiError;
use reqwest::{get, Client};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

static NORDIGEN_REFRESH: &str = "https://ob.nordigen.com/api/v2/token/refresh/";
static NORDIGEN_ACCOUNTS: &str = "https://ob.nordigen.com/api/v2/accounts/";

#[derive(Clone)]
pub enum FiatCurrency {
    EURO,
    USD,
}

impl FiatCurrency {
    pub fn to_string(&self) -> String {
        match self {
            Self::EURO => "EUR",
            Self::USD => "USD",
        }
        .to_string()
    }

    pub fn from_string(string: String) -> Self {
        match string.as_str() {
            "EUR" => Self::EURO,
            "USD" => Self::USD,
            x => todo!("Currency {} not supported yet", x),
        }
    }
}

pub struct NordigenCache {
    pub max_age: Duration,
    access_token: Arc<Mutex<String>>,
    balance_table: Arc<Mutex<HashMap<String, (SystemTime, f32, FiatCurrency)>>>,
}

impl NordigenCache {
    pub fn new(max_age_hours: u64) -> Self {
        Self {
            max_age: Duration::from_secs(max_age_hours * 3600),
            access_token: Arc::new(Mutex::new(String::new())),
            balance_table: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, account_id: &String) -> Option<(f32, FiatCurrency)> {
        match self.balance_table.lock().unwrap().get(account_id) {
            None => None,
            Some((time, balance, currency)) => {
                if SystemTime::now().duration_since(*time).unwrap() > self.max_age {
                    None
                } else {
                    Some((*balance, (*currency).clone()))
                }
            }
        }
    }

    pub fn set(&self, account_id: String, balance: f32, currency: FiatCurrency) {
        self.balance_table
            .lock()
            .unwrap()
            .insert(account_id, (SystemTime::now(), balance, currency));
    }

    pub fn set_access_token(&self, access_token: String) {
        *self.access_token.lock().unwrap() = access_token;
    }

    pub fn get_access_token(&self) -> String {
        self.access_token.lock().unwrap().clone()
    }
}

enum BalanceAccountResult {
    Ok((f32, FiatCurrency)),
    AccessTokenExpired,
}

async fn get_new_access_token(refresh_token: &String) -> Result<String, ApiError> {
    println!("Refresh nordigen access token");
    let client = Client::new();
    let mut map = HashMap::new();
    map.insert("refresh".to_string(), refresh_token);
    let result = client
        .post(NORDIGEN_REFRESH)
        .json(&map)
        .send()
        .await?
        .json::<Value>()
        .await?;

    match result.get("access") {
        None => Err(ApiError::new(&format!(
            "Nordigen refresh token expired: {}\n{:?}",
            refresh_token, result
        ))),
        Some(v) => Ok(v.as_str().unwrap().to_string()),
    }
}

async fn get_balance_of_account(
    access_token: &String,
    account_id: &String,
) -> Result<BalanceAccountResult, ApiError> {
    let url = format!("{}{}/balances/", NORDIGEN_ACCOUNTS, account_id);
    let client = Client::new();
    let result = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(Value::String(v)) = result.get("summary") {
        if v == "Invalid token" {
            return Ok(BalanceAccountResult::AccessTokenExpired);
        }
    }

    if let Some(Value::Number(v)) = result.get("status_code") {
        if v.as_i64().unwrap() == 404 {
            println!("Error! Nordigen Account {} not found!", account_id);
            return Ok(BalanceAccountResult::Ok((0.0, FiatCurrency::USD)));
        }
    }

    let balance_details = result.get("balances").unwrap()[0]
        .get("balanceAmount")
        .unwrap();
    let balance_currency = balance_details.get("currency").unwrap().as_str().unwrap();
    let balance_amount = balance_details.get("amount").unwrap().as_str().unwrap();

    Ok(BalanceAccountResult::Ok((
        balance_amount.parse::<f32>().unwrap(),
        FiatCurrency::from_string(balance_currency.to_string()),
    )))
}

pub async fn get_assets_of_nordigen_account(
    nordigen_cache: &NordigenCache,
    account: &NordigenAccount,
) -> Result<Vec<GenericAsset>, ApiError> {
    let cache_hit = nordigen_cache.get(&account.account_id);
    if let Some((balance, currency)) = cache_hit {
        return Ok(vec![GenericAsset::new(
            0.0,
            currency.to_string(),
            account.name.clone(),
            balance,
            get_token_price(&"EUR".to_string()).await?,
        )]);
    }

    let access_token = if nordigen_cache.get_access_token().is_empty() {
        get_new_access_token(&account.refresh_token).await?
    } else {
        nordigen_cache.get_access_token()
    };

    nordigen_cache.set_access_token(access_token.clone());

    let balance = get_balance_of_account(&access_token, &account.account_id).await?;

    if let BalanceAccountResult::Ok((balance, currency)) = balance {
        nordigen_cache.set(account.account_id.clone(), balance, currency.clone());
        return Ok(vec![GenericAsset::new(
            0.0,
            currency.to_string(),
            account.name.clone(),
            balance,
            1.19,
        )]);
    }

    let access_token = get_new_access_token(&account.refresh_token).await?;
    nordigen_cache.set_access_token(access_token.clone());

    let balance = get_balance_of_account(&access_token, &account.account_id).await?;

    if let BalanceAccountResult::Ok((balance, currency)) = balance {
        nordigen_cache.set(account.account_id.clone(), balance, currency.clone());
        return Ok(vec![GenericAsset::new(
            0.0,
            currency.to_string(),
            account.name.clone(),
            balance,
            1.13, // TODO not hard coded
        )]);
    }

    Ok(vec![])
}
