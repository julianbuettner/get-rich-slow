use super::super::cryptoprice::get_token_price;
use super::super::error::ApiError;
use super::abi::{get_benqi_abi, get_benqi_comptroller_abi};
use super::defiasset::DefiAsset;
use regex::Regex;
use web3::contract::Contract as Web3Contract;
use web3::transports::Http;
use web3::types::{Address, H160};
use web3::Web3;

static DIGITS_AVAX_AND_BENQI_FOR_PENDING_REWARDS: u8 = 18;

fn get_underlaying_name(benqi_token: &String) -> Result<String, ApiError> {
    let pattern = Regex::new(r"qi(?P<underlaying>[A-Z]{3,4})").unwrap();
    let capture = pattern.captures(&benqi_token);
    if capture.is_none() {
        return Err(ApiError::new(&format!(
            "No valid Benqi Token: {}",
            benqi_token
        )));
    }
    Ok(capture.unwrap()["underlaying"].to_string())
}

async fn get_benqi_token_price(benqi_token: &String) -> Result<f32, ApiError> {
    get_token_price(&get_underlaying_name(benqi_token)?).await
}

pub async fn get_benqi_assets(
    web3: &Web3<Http>,
    wallet_address: &Address,
    contract_addresses: &Vec<Address>,
) -> Result<Vec<DefiAsset>, ApiError> {
    let mut res = Vec::new();

    for contract_address in contract_addresses.iter() {
        let smart = Web3Contract::new(web3.eth(), *contract_address, get_benqi_abi());

        let balance: web3::types::U256 = smart
            .query(
                "balanceOf",
                (*wallet_address,),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await?;

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
            .await?;
        let decimals = decimals as f64;

        let balance = balance.as_u128();
        let balance_float = balance as f64 / (10 as f64).powf(decimals);

        let exchange_rate: web3::types::U256 = smart
            .query(
                "exchangeRateStored",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await?;
        let exchange_rate_float = exchange_rate.as_u128() as f64 * (10. as f64).powf(-28.);

        let apy: web3::types::U256 = smart
            .query(
                "supplyRatePerTimestamp",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await?;
        let apy_float = apy.as_u128() as f64 * (10. as f64).powf(-18.) * 3600. * 24. * 365.;

        let symbol: String = smart
            .query("symbol", (), None, web3::contract::Options::default(), None)
            .await?;

        res.push(DefiAsset::new(
            apy_float as f32,
            get_underlaying_name(&symbol)?,
            balance_float as f32 * exchange_rate_float as f32,
            get_benqi_token_price(&symbol).await?,
            symbol,
        ));
    }

    // Check for outstanding QI or AVAX
    // reward type 0 = QI, 1 = AVAX
    let address: Address = H160([
        0x48, 0x6A, 0xf3, 0x95, 0x19, 0xB4, 0xDc, 0x9a, 0x7f, 0xCc, 0xd3, 0x18, 0x21, 0x73, 0x52,
        0x83, 0x0E, 0x8A, 0xD9, 0xb4,
    ]);
    /*let address: Address = H160(
        [
            0xd3, 0x8a, 0x19, 0x10, 0x05, 0x30, 0xb9, 0x9c, 0x3b, 0x84,
            0xcc, 0xa9, 0x71, 0xdf, 0xd9, 0x6b, 0xd5, 0x57, 0xaa, 0x91,
        ]
    );*/
    let smart = Web3Contract::new(web3.eth(), address, get_benqi_comptroller_abi());
    let outstanding_qi: web3::types::U256 = smart
        .query(
            "getBlockTimestamp", // rewardAccrued
            (),                  // (0 as u16, *wallet_address),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
        .unwrap();
    let outstanding_avax: web3::types::U256 = smart
        .query(
            "getBlockTimestamp",
            (), // (1 as u16, *wallet_address),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
        .unwrap();
    let outstanding_qi_balance = outstanding_qi.as_u128() as f64
        * (10. as f64).powf(-(DIGITS_AVAX_AND_BENQI_FOR_PENDING_REWARDS as f64));
    let outstanding_avax_balance = outstanding_avax.as_u128() as f64
        * (10. as f64).powf(-(DIGITS_AVAX_AND_BENQI_FOR_PENDING_REWARDS as f64));

    if outstanding_qi.as_u128() > 0 {
        res.push(DefiAsset::new(
            0.,
            "QI".to_string(),
            outstanding_qi_balance as f32,
            get_token_price(&"QI".to_string()).await?,
            "QI pending on Benqi".to_string(),
        ));
    }
    if outstanding_avax.as_u128() > 0 {
        res.push(DefiAsset::new(
            0.,
            "AVAX".to_string(),
            outstanding_avax_balance as f32,
            get_token_price(&"AVAX".to_string()).await?,
            "AVAX pending on Benqi".to_string(),
        ));
    }
    Ok(res)
}
