use super::account::ScalableAccount;
use super::asset::GenericAsset;
use super::cryptoprice::get_token_price;
use super::error::ApiError;
use reqwest::Client;
use serde_json::{Value, from_str};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

static LOGIN_URL: &str = "https://de.scalable.capital/actions/login";
static PORTFOLIO_URL: &str = "https://de.scalable.capital/cockpit/graphql";

pub struct ScalableCache {
    access_token_map: Arc<Mutex<HashMap<String, String>>>,
}

impl ScalableCache {
    pub fn new() -> Self {
        Self {
            access_token_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, email: &String) -> Option<String> {
        match self.access_token_map.lock().unwrap().get(email) {
            None => None,
            Some(v) => Some(v.clone()),
        }
    }

    pub fn set(&self, email: String, s: String) {
        self.access_token_map.lock().unwrap().insert(email, s);
    }
}

fn get_graphql_request(person_id: String) -> Value {
    // TODO use GraphQL
    let data = r#"
    [
        {
        "operationName": "getBrokerPortfolios",
        "variables": {
            "personId": "PERSON_ID"
        },
        "query": "query getBrokerPortfolios($personId: ID!) {\n  account(id: $personId) {\n    id\n    ...BrokerPortfoliosOnAccountFragment\n    __typename\n  }\n}\n\nfragment BrokerPortfoliosOnAccountFragment on Account {\n  brokerPortfolios {\n    id\n    totalSavingsPlanAmount\n    numberOfPendingOrders\n    postOnboardingInfo {\n      id\n      allStepsCompleted\n      steps {\n        status\n        type\n        __typename\n      }\n      __typename\n    }\n    valuation {\n      id\n      valuation\n      cryptoValuation\n      time\n      timeWeightedReturnByTimeframe {\n        timeframe\n        performance\n        simpleAbsoluteReturn\n        __typename\n      }\n      __typename\n    }\n    selectedOffer {\n      id\n      __typename\n    }\n    __typename\n  }\n  __typename\n}\n"
        }
    ]
    "#;
    serde_json::from_str(&data.replace("PERSON_ID", &person_id)).unwrap()
}

fn stock_crypto_balance_from_json(v: Value) -> Result<(f32, f32), ApiError> {
    let default_error = ApiError::new(&String::from("Unexpected Scalable GraphQL response format"));

    let first = &v[0];
    let data = first.get("data");
    if data.is_none() {
        return Err(default_error);
    }
    let account = data.unwrap().get("account");
    if account.is_none() {
        return Err(default_error);
    }
    let broker = account.unwrap().get("brokerPortfolios");
    if broker.is_none() {
        return Err(default_error);
    }
    let portfolio = &broker.unwrap()[0];
    let valuation = portfolio.get("valuation");
    if valuation.is_none() {
        return Err(default_error);
    }
    let valuation = valuation.unwrap();

    let stock_valuation = valuation.get("valuation");
    let crypto_valuation = valuation.get("cryptoValuation");

    if stock_valuation.is_none() {
        return Err(default_error);
    }
    if crypto_valuation.is_none() {
        return Err(default_error);
    }
    Ok(
        (
            stock_valuation.unwrap().as_f64().unwrap() as f32,
            crypto_valuation.unwrap().as_f64().unwrap() as f32,
        )
    )
}

fn get_person_id_from_access_token(access_token: String) -> Result<String, ApiError> {
    // Scalable uses a special format, not supported by the jwt lib.
    // Therefore we have to get our information manually.

    // header.payload.signature

    let mut split = access_token.split(".");
    split.next(); // header
    let payload = split.next().unwrap().clone();
    let bytes = base64::decode(&payload.to_string()).unwrap();
    let json_text = std::str::from_utf8(&bytes).unwrap();

    let value: Value = from_str(json_text).unwrap();

    match value.get("person_id") {
        None => Err(ApiError::new(&String::from("person_id not found in Scalabe JWT"))),
        Some(v) => Ok(v.as_str().unwrap().to_string())
    }
}

async fn get_access_token(email: &String, password: &String) -> Result<String, ApiError> {
    let mut map = HashMap::new();
    map.insert("username".to_string(), email.clone());
    map.insert("password".to_string(), password.clone());

    println!("Perform Scalable login");
    let client = Client::new();
    let result = client
        .post(LOGIN_URL)
        .json(&map)
        .send()
        .await?
        .json::<Value>()
        .await?;

    match result.get(0) {
        None => Err(ApiError::new(&format!("Scalable Capital login failed."))),
        Some(v) => match v.get("accessToken") {
            None => Err(ApiError::new(&format!("Scalable Capital login failed."))),
            Some(x) => Ok(x.as_str().unwrap().to_string()),
        },
    }
}

enum AssetResult {
    AccessExpired,
    Assets(Vec<GenericAsset>),
}

async fn get_assets(access_token: &String, account_name: &String) -> Result<AssetResult, ApiError> {
    let person_id = get_person_id_from_access_token(access_token.clone())?;
    let client = Client::new();
    let result = client
        .post(PORTFOLIO_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&get_graphql_request(person_id))
        .send()
        .await?
        .json::<Value>()
        .await?;

    let (stocks, crypto) = stock_crypto_balance_from_json(result)?;

    let eur_price = get_token_price(&"EUR".to_string()).await?;
    Ok(AssetResult::Assets(vec![
        GenericAsset::new(
            0.0,
            "Stocks on Scalable".to_string(),
            format!("{} | Stocks", account_name),
            stocks,
            eur_price,
        ),
        GenericAsset::new(
            0.0,
            "Crypto on Scalable".to_string(),
            format!("{} | Crypto", account_name),
            crypto,
            eur_price,
        ),
    ]))
}

pub async fn get_assets_of_scalable_account(
    cache: &ScalableCache,
    account: &ScalableAccount,
) -> Result<Vec<GenericAsset>, ApiError> {
    let cached_access_token = cache.get(&account.email);
    let access_token = match cached_access_token {
        None => { 
            let access = get_access_token(&account.email, &account.password).await?;
            cache.set(account.email.clone(), access.clone());
            access
        },
        Some(v) => v,
    };

    match get_assets(&access_token, &account.name).await? {
        AssetResult::AccessExpired => (),
        AssetResult::Assets(a) => return Ok(a),
    }

    let access_token = get_access_token(&account.email, &account.password).await?;
    cache.set(account.email.clone(), access_token.clone());

    match get_assets(&access_token, &account.name).await? {
        AssetResult::AccessExpired => Err(ApiError::new(
            &"Scalable access expired immediately".to_string(),
        )),
        AssetResult::Assets(a) => Ok(a),
    }
}
