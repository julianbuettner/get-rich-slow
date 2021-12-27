use serde::Deserialize;
use serde_yaml::from_str;
use std::collections::HashMap;
use std::{env, fs};

#[derive(Deserialize, Debug, Clone)]
pub struct Account {
    pub kind: String,
    #[serde(rename = "api-key")]
    pub api_key: Option<String>, // Etoro, Kraken, Binance
    #[serde(rename = "api-secret")]
    pub api_secret: Option<String>, // Kraken, Binance
    pub address: Option<String>, // Crypto
}

#[derive(Deserialize, Debug, Clone)]
pub struct Fund {
    pub name: String,
    pub icon: Option<String>,
    pub accounts: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub accounts: HashMap<String, Account>,
    pub funds: Vec<Fund>,
    #[serde(rename = "smart-contracts")]
    pub smart_contracts: HashMap<String, HashMap<String, Vec<String>>>,
    pub nodes: HashMap<String, String>,
    pub clients: Option<Vec<String>>,
    pub port: u16,
    #[serde(rename = "listen-address")]
    pub listen_address: String,
}

pub fn read_config() -> Configuration {
    let home = env::var("HOME").expect("HOME variable undefined");
    let content = fs::read_to_string(&format!("{}/.get-rich-slow.yaml", home))
        .expect("Failed to read ~/.get-rich-slow.yaml");
    from_str(&content).expect("Failed to parse ~/.get-rich-slow.yaml")
}
