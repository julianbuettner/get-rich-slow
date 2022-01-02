use super::ethereum::EthereumChain;
use web3::types::Address;

#[derive(Clone)]
pub struct EthereumAccount {
    pub name: String,
    pub chain: EthereumChain,
    pub wallet_address: Address,
}

#[derive(Clone)]
pub struct KrakenAccount {
    pub name: String,
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Clone)]
pub enum Account {
    Ethereum(EthereumAccount),
    Etoro { name: String, api_key: String },
    Kraken(KrakenAccount),
}
