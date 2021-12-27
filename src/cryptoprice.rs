use super::error::ApiError;
use reqwest::get;
use serde_json::Value;

pub async fn get_token_price(symbol: &String) -> Result<f32, ApiError> {
    let mut symbol = symbol.clone();

    // Special symbol handling
    match symbol.as_str() {
        "BUSD" => return Ok(1.0),
        "BTCB" => symbol = "BTC".to_string(),
        _ => (),
    }

    let result = get(format!(
        "https://api.binance.com/api/v1/ticker/24hr?symbol={}BUSD",
        symbol
    ))
    .await?
    .json::<Value>()
    .await?;
    let price_string = result.get("lastPrice");
    if price_string.is_none() {
        return Err(ApiError::new(&format!(
            "Invalid JSON format when looking up price for {}",
            symbol
        )));
    }
    let price_string = price_string.unwrap().as_str().unwrap();

    Ok(price_string.parse::<f32>()?)
}
