use super::account::KrakenAccount;
use super::asset::GenericAsset;
use super::domainconfig::DomainConfig;
use super::error::ApiError;
use kraken_client::Client;
use regex::Regex;
use std::collections::hash_map::HashMap;

fn trading_symbol_lookup(symbol: &str) -> Result<String, ApiError> {
    let pattern = Regex::new(r"(?P<trading_symbol>[A-Z0-9]{2,5})(\.(S|P))?").unwrap();
    let capture = pattern.captures(&symbol);
    if capture.is_none() {
        return Err(ApiError::new(&format!(
            "No valid Kraken Symbol: {}",
            symbol
        )));
    }
    Ok(capture.unwrap()["trading_symbol"].to_string())
}

async fn kraken_price_hashmap(
    client: &Client,
    symbols: &Vec<String>,
) -> Result<HashMap<String, f32>, ApiError> {
    let tickers = symbols
        .iter()
        .filter(|x| *x != "ZUSD")
        .filter(|x| *x != "ETH2")
        .filter(|x| *x != "ZEUR")
        .map(|x| {
            if x.starts_with("X") && (x != "XTZ") {
                format!("{}ZUSD", x)
            } else {
                format!("{}USD", x)
            }
        })
        .collect::<Vec<String>>()
        .join(",");
    let tickers = tickers + ",ETH2.SETH,XETHZUSD,ZEURZUSD";
    let response = client.get_tickers(&tickers).send().await?;
    let mut result = HashMap::new();
    result.insert("ZUSD".to_string(), 1.0);
    let x_pattern = Regex::new(r"(?P<trading_symbol>[A-Z]{2,4})ZUSD").unwrap();
    let pattern = Regex::new(r"(?P<trading_symbol>[A-Z]{2,4})USD").unwrap();

    let mut eth2eth = None;
    for (trading_pair_symbol, ticker) in response.iter() {
        let price = ticker.c[0].parse::<f32>().unwrap();
        if trading_pair_symbol == "ETH2.SETH" {
            eth2eth = Some(price);
            continue;
        }

        let x_capture = x_pattern.captures(&trading_pair_symbol);
        let capture = pattern.captures(&trading_pair_symbol);
        if capture.is_none() && x_capture.is_none() {
            return Err(ApiError::new(&format!(
                "No valid Kraken Pair Symbol: {}",
                trading_pair_symbol
            )));
        }
        let key = if x_capture.is_some() {
            x_capture.unwrap()["trading_symbol"].to_string()
        } else {
            capture.unwrap()["trading_symbol"].to_string()
        };
        result.insert(key, price);
    }
    if eth2eth.is_none() {
        return Err(ApiError::new(&"Could not look up ETH2 price!".to_string()));
    }
    result.insert(
        "ETH2".to_string(),
        eth2eth.unwrap() * result.get("XETH").unwrap(),
    );
    Ok(result)
}

async fn get_apy(_client: &Client, symbol: &str) -> Result<f32, ApiError> {
    let apr = match symbol {
        "DOT.S" => 0.12,
        "ETH2.S" => 0.03,
        _ => 0.0,
    };
    Ok(apr)
}

pub async fn get_assets_of_kraken_account(
    _domainconfig: &DomainConfig,
    account: &KrakenAccount,
) -> Result<Vec<GenericAsset>, ApiError> {
    let client = Client::new(account.api_key.as_str(), account.api_secret.as_str());

    let response = client.get_account_balance().send().await?;

    let response = response
        .iter()
        .map(|(key, value)| (key.to_string(), value.parse::<f32>().unwrap()))
        .filter(|(_, value)| *value > 0.000000001)
        .collect::<Vec<(String, f32)>>();

    let symbols = response
        .iter()
        .map(|(key, _)| trading_symbol_lookup(key).ok().unwrap())
        .collect::<Vec<String>>();

    let kraken_hashmap = kraken_price_hashmap(&client, &symbols).await?;

    let mut result = Vec::new();
    for (key, value) in response {
        let trading_symbol = trading_symbol_lookup(&key.to_string())?;
        result.push(GenericAsset::new(
            get_apy(&client, &key).await?,
            trading_symbol.clone(),
            format!("{} on Kraken", key),
            value,
            *kraken_hashmap.get(&trading_symbol).unwrap_or(&0.0),
        ))
    }

    Ok(result)
}
