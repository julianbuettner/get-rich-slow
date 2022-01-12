use super::account::{Account, EthereumAccount, KrakenAccount};
use super::blockchain::EthDefiToken;
use super::config::Configuration;
use super::ethereum::{EthereumChain, EthereumNode};
use hex::decode_to_slice;
use std::collections::HashMap;
use web3::types::{Address, H160};

pub struct Fund {
    pub name: String,
    pub icon: Option<String>,
    pub accounts: Vec<Account>,
    pub target_size: Option<f32>,
}

pub struct DomainConfig {
    pub funds: Vec<Fund>,
    pub smart_contracts: Vec<(EthereumChain, EthDefiToken, Address)>,
    pub eth_nodes: Vec<EthereumNode>,
    pub client_whitelist: Option<Vec<String>>,
    pub port: u16,
    pub listen_address: String,
}

fn hex_to_address(hex: &String) -> Address {
    let mut slice: [u8; 20] = [0; 20];
    decode_to_slice(hex, &mut slice as &mut [u8]).unwrap();
    H160(slice)
}

fn blockchain_identifier_to_enum(identifier: &str) -> EthereumChain {
    match identifier {
        "bsc" => EthereumChain::BinanceSmartChain,
        "avalanche-c" => EthereumChain::AvalancheC,
        "ethereum" => EthereumChain::Ethereum,
        "moonriver" => EthereumChain::Moonriver,
        "moonbeam" => EthereumChain::Moonbeam,
        e => panic!("Unexpected blockchain identifier: {}", e),
    }
}

impl DomainConfig {
    pub fn from_config(config: Configuration) -> Self {
        let mut accounts = HashMap::new();
        for (name, account_config) in config.accounts.iter() {
            let account = match account_config.kind.as_str() {
                "bsc" => Account::Ethereum(EthereumAccount {
                    name: name.clone(),
                    chain: EthereumChain::BinanceSmartChain,
                    wallet_address: hex_to_address(
                        &account_config
                            .address
                            .clone()
                            .expect("Account of type bsc requires address"),
                    ),
                }),
                "avalanche-c" => Account::Ethereum(EthereumAccount {
                    name: name.clone(),
                    chain: EthereumChain::AvalancheC,
                    wallet_address: hex_to_address(
                        &account_config
                            .address
                            .clone()
                            .expect("Account of type bsc requires address"),
                    ),
                }),
                "kraken" => Account::Kraken(KrakenAccount {
                    name: name.clone(),
                    api_key: account_config
                        .api_key
                        .clone()
                        .expect("Kraken requires api-key"),
                    api_secret: account_config
                        .api_secret
                        .clone()
                        .expect("Kraken requires api-secret"),
                }),
                "moonbeam" => Account::Ethereum(EthereumAccount {
                    name: name.clone(),
                    chain: EthereumChain::Moonbeam,
                    wallet_address: hex_to_address(
                        &account_config
                            .address
                            .clone()
                            .expect("Account of type moonbeam requires address")
                    ),
                }),
                x => panic!("Invalid account type: {}", x),
            };
            accounts.insert(name.clone(), account);
        }

        let mut funds = Vec::new();
        for config_fund in config.funds.iter() {
            let mut fund_accounts: Vec<Account> = Vec::new();
            for account_identifier in config_fund.accounts.iter() {
                fund_accounts.push(
                    accounts
                        .get(account_identifier)
                        .expect(&format!(
                            "Account with identifier {} not found",
                            account_identifier
                        ))
                        .clone(),
                )
            }
            funds.push(Fund {
                name: config_fund.name.clone(),
                icon: config_fund.icon.clone(),
                accounts: fund_accounts,
                target_size: config_fund.target_size,
            })
        }

        let mut smart_contracts: Vec<(EthereumChain, EthDefiToken, Address)> = Vec::new();
        for (chain_identifier, defi_platforms) in config.smart_contracts.iter() {
            let chain = blockchain_identifier_to_enum(chain_identifier.as_str());
            for (defi_platform_name, contract_address_strings) in defi_platforms.iter() {
                let defi_platform = match defi_platform_name.as_str() {
                    "bep20" => EthDefiToken::Bep20,
                    "alpaca" => EthDefiToken::Alpaca,
                    "venus" => EthDefiToken::Venus,
                    "benqi" => EthDefiToken::Benqi,
                    e => panic!("Unexpected DeFi platform identifier: {}", e),
                };
                contract_address_strings
                    .iter()
                    .map(|x| {
                        smart_contracts.push((
                            chain.clone(),
                            defi_platform.clone(),
                            hex_to_address(x),
                        ))
                    })
                    .for_each(drop);
            }
        }

        let nodes = config
            .nodes
            .iter()
            .map(|(name, http_address)| EthereumNode {
                chain: blockchain_identifier_to_enum(name.as_str()),
                web3: web3::Web3::new(web3::transports::Http::new(http_address.as_str()).unwrap()),
            })
            .collect::<Vec<EthereumNode>>();

        Self {
            funds: funds,
            smart_contracts: smart_contracts,
            eth_nodes: nodes,
            client_whitelist: config.clients,
            port: config.port,
            listen_address: config.listen_address,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hex_string_parsing() {
        assert_eq!(
            web3::types::H160([
                0x7c, 0x9e, 0x73, 0xd4, 0xc7, 0x1d, 0xae, 0x56, 0x4d, 0x41, 0xf7, 0x8d, 0x56, 0x43,
                0x9b, 0xb4, 0xba, 0x87, 0x59, 0x2f
            ]),
            hex_to_address(&"7c9e73d4c71dae564d41f78d56439bb4ba87592f".to_string())
        )
    }
}
