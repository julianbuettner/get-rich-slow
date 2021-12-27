use super::super::cryptoprice::get_token_price;
use super::super::error::ApiError;
use super::abi::{get_alapca_interest_rate_model_abi, get_alpaca_abi, get_alpaca_config_abi};
use super::defiasset::DefiAsset;
use regex::Regex;
use web3::contract::Contract as Web3Contract;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

fn get_underlaying_name(ibtoken: &String) -> Result<String, ApiError> {
    let pattern = Regex::new(r"ib(?P<underlaying>[A-Z]{3,4})").unwrap();
    let capture = pattern.captures(&ibtoken);
    if capture.is_none() {
        return Err(ApiError::new(&format!(
            "No valid Alpaca ibToken: {}",
            ibtoken
        )));
    }
    Ok(capture.unwrap()["underlaying"].to_string())
}

async fn get_ibtoken_price(ibtoken: &String) -> Result<f32, ApiError> {
    get_token_price(&get_underlaying_name(ibtoken)?).await
}

// https://bscscan.com/address/0xadcfbf2e8470493060fbe0a0afac66d2cb028e9c#readContract
// https://docs.alpacafinance.org/our-protocol-1/global-parameters#interest-rate-model
struct InterestModel {
    ceil_slope_1: f64,
    ceil_slope_2: f64,
    ceil_slope_3: f64,
    max_interest_slope_1: f64,
    max_interest_slope_2: f64,
    max_interest_slope_3: f64,
}

impl InterestModel {
    pub fn new(
        ceil_slope_1: f64,
        ceil_slope_2: f64,
        ceil_slope_3: f64,
        max_interest_slope_1: f64,
        max_interest_slope_2: f64,
        max_interest_slope_3: f64,
    ) -> Self {
        Self {
            ceil_slope_1: ceil_slope_1,
            ceil_slope_2: ceil_slope_2,
            ceil_slope_3: ceil_slope_3,
            max_interest_slope_1: max_interest_slope_1,
            max_interest_slope_2: max_interest_slope_2,
            max_interest_slope_3: max_interest_slope_3,
        }
    }

    pub fn get_borrow_rate_from_usage(&self, usage: f64) -> f64 {
        assert!(usage <= 1.);
        if usage < self.ceil_slope_1 {
            let slope_ratio = usage / self.ceil_slope_1;
            return slope_ratio * self.max_interest_slope_1;
        }
        if usage < self.ceil_slope_2 {
            let slope_ratio = (usage - self.ceil_slope_1) / (self.ceil_slope_2 - self.ceil_slope_1);
            return slope_ratio * (self.max_interest_slope_2 - self.max_interest_slope_1)
                + self.max_interest_slope_1;
        }
        if usage <= self.ceil_slope_3 {
            let slope_ratio = (usage - self.ceil_slope_2) / (self.ceil_slope_3 - self.ceil_slope_2);
            return slope_ratio * (self.max_interest_slope_3 - self.max_interest_slope_2)
                + self.max_interest_slope_2;
        }
        panic!("Alpaca interest model with more than three slopes");
    }
}

async fn get_interest_model(contract: &Web3Contract<Http>) -> Result<InterestModel, ApiError> {
    let cs1: web3::types::U256 = contract
        .query(
            "CEIL_SLOPE_1",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await?;
    let cs2: web3::types::U256 = contract
        .query(
            "CEIL_SLOPE_2",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await?;
    let cs3: web3::types::U256 = contract
        .query(
            "CEIL_SLOPE_3",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await?;
    let ms1: web3::types::U256 = contract
        .query(
            "MAX_INTEREST_SLOPE_1",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await?;
    let ms2: web3::types::U256 = contract
        .query(
            "MAX_INTEREST_SLOPE_2",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await?;
    let ms3: web3::types::U256 = contract
        .query(
            "MAX_INTEREST_SLOPE_3",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await?;
    Ok(InterestModel::new(
        (cs1.as_u128() / 10_000_000_000_000_000) as f64 / 10_000.,
        (cs2.as_u128() / 10_000_000_000_000_000) as f64 / 10_000.,
        (cs3.as_u128() / 10_000_000_000_000_000) as f64 / 10_000.,
        (ms1.as_u128() / 100_000_000_000_000) as f64 / 10_000.,
        (ms2.as_u128() / 100_000_000_000_000) as f64 / 10_000.,
        (ms3.as_u128() / 100_000_000_000_000) as f64 / 10_000.,
    ))
}

pub async fn get_alpaca_assets(
    web3: &Web3<Http>,
    wallet_address: &Address,
    contract_addresses: &Vec<Address>,
) -> Result<Vec<DefiAsset>, ApiError> {
    let mut res = Vec::new();

    for contract_address in contract_addresses.iter() {
        let smart = Web3Contract::new(web3.eth(), *contract_address, get_alpaca_abi());

        let balance: web3::types::U256 = smart
            .query(
                "balanceOf",
                (*wallet_address,),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await
            .expect("A");

        if balance.as_u128() == 0 {
            continue;
        }

        let decimals: u8 = smart
            .query(
                "decimals",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await
            .expect("B");
        let decimals = decimals as f64;

        let total_tokens: web3::types::U256 = smart
            .query(
                "totalToken",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await
            .expect("C");
        let total_supply: web3::types::U256 = smart
            .query(
                "totalSupply",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await
            .expect("D");
        let ratio = total_tokens.as_u128() as f64 / total_supply.as_u128() as f64;

        let balance = balance.as_u128();
        let balance_float = balance as f64 / (10 as f64).powf(decimals);

        let symbol: String = smart
            .query("symbol", (), None, web3::contract::Options::default(), None)
            .await
            .expect("E");

        let total_supply: web3::types::U256 = smart
            .query(
                "totalToken",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await
            .expect("E");
        let total_borrow: web3::types::U256 = smart
            .query(
                "vaultDebtVal",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await
            .expect("E");
        let usage = total_borrow.as_u128() as f64 / total_supply.as_u128() as f64;

        let data: Address = smart
            .query("config", (), None, web3::contract::Options::default(), None)
            .await?;
        let smart_config = Web3Contract::new(web3.eth(), data, get_alpaca_config_abi());
        let interest_model_address: Address = smart_config
            .query(
                "interestModel",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await?;

        let smart_interest_model = Web3Contract::new(
            web3.eth(),
            interest_model_address,
            get_alapca_interest_rate_model_abi(),
        );
        let interest_model = get_interest_model(&smart_interest_model).await?;

        let borrow_rate = interest_model.get_borrow_rate_from_usage(usage);
        let lending_fee_rate = 0.81;
        let apy = borrow_rate * usage * lending_fee_rate;
        res.push(DefiAsset::new(
            apy as f32,
            get_underlaying_name(&symbol)?,
            balance_float as f32 * ratio as f32,
            get_ibtoken_price(&symbol).await?,
            symbol,
        ));
    }

    Ok(res)
}
