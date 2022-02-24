mod abi;
mod alpaca;
mod benqi;
mod defiasset;
mod venus;

use super::account::EthereumAccount;
use super::cryptoprice::get_token_price;
use super::domainconfig::DomainConfig;
use super::error::ApiError;
use super::ethereum::{EthereumChain, EthereumNode};
use web3::types::Address;

#[derive(Clone, PartialEq)]
pub enum EthDefiToken {
    Bep20,
    Alpaca,
    Venus,
    Benqi,
}

async fn get_native_asset(
    web3: &web3::Web3<web3::transports::Http>,
    account: &EthereumAccount,
) -> Result<Vec<defiasset::DefiAsset>, ApiError> {
    let balance = web3.eth().balance(account.wallet_address, None).await?;
    if balance.as_u128() == 0 {
        return Ok(Vec::new());
    }
    let digits = match account.chain {
        _ => 18,
    };
    let b = balance.as_u128() as f64 / (10 as f64).powf(digits as f64);
    let trading_symbol = account.chain.to_trading_symbol();
    let trading_price = get_token_price(&trading_symbol).await?;
    Ok(vec![defiasset::DefiAsset::new(
        0.,
        trading_symbol,
        b as f32,
        trading_price,
        account.chain.to_str().to_string(),
    )])
}

pub async fn get_assets_of_ethereum_account(
    domainconfig: &DomainConfig,
    account: &EthereumAccount,
) -> Result<Vec<defiasset::DefiAsset>, ApiError> {
    let nodes_for_chain: Vec<&EthereumNode> = domainconfig
        .eth_nodes
        .iter()
        .filter(|x| x.chain == account.chain)
        .collect();
    if nodes_for_chain.is_empty() {
        return Err(ApiError::new(&format!(
            "No node configured for blockchain {}",
            account.chain.to_str()
        )));
    }

    let contracts_on_account_chain: Vec<(EthDefiToken, Address)> = domainconfig
        .smart_contracts
        .iter()
        .filter(|x| x.0 == account.chain)
        .map(|x| (x.1.clone(), x.2))
        .collect();

    let venus_contracts: Vec<Address> = contracts_on_account_chain
        .iter()
        .filter(|x| x.0 == EthDefiToken::Venus)
        .map(|x| x.1)
        .collect::<Vec<Address>>();

    let alpaca_contracts: Vec<Address> = contracts_on_account_chain
        .iter()
        .filter(|x| x.0 == EthDefiToken::Alpaca)
        .map(|x| x.1)
        .collect::<Vec<Address>>();

    let benqi_contracts: Vec<Address> = contracts_on_account_chain
        .iter()
        .filter(|x| x.0 == EthDefiToken::Benqi)
        .map(|x| x.1)
        .collect::<Vec<Address>>();

    let mut one_node_worked = false;
    let mut assets = Vec::new();
    for node in nodes_for_chain {
        let native = get_native_asset(&node.web3, &account).await;
        if native.is_err() {
            continue;
        }
        let v = if account.chain == EthereumChain::BinanceSmartChain {
            venus::get_venus_assets(&node.web3, &account.wallet_address, &venus_contracts).await
        } else {
            Ok(Vec::new())
        };
        if v.is_err() {
            continue;
        }
        let a = if account.chain == EthereumChain::BinanceSmartChain {
            alpaca::get_alpaca_assets(&node.web3, &account.wallet_address, &alpaca_contracts).await
        } else {
            Ok(Vec::new())
        };
        if a.is_err() {
            continue;
        }
        let b = if account.chain == EthereumChain::AvalancheC {
            benqi::get_benqi_assets(&node.web3, &account.wallet_address, &benqi_contracts).await
        } else {
            Ok(Vec::new())
        };
        if b.is_err() {
            continue;
        }
        one_node_worked = true;
        assets.append(&mut native.ok().unwrap());
        assets.append(&mut v.ok().unwrap());
        assets.append(&mut a.ok().unwrap());
        assets.append(&mut b.ok().unwrap());
    }
    if !one_node_worked {
        return Err(ApiError::new(&format!(
            "All nodes for blockchain {} failed for at least one contract",
            account.chain.to_str()
        )));
    }

    Ok(assets)
}
