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

static BENQI_COMPTROLLER_ADDRESS: Address = H160([
    0x48, 0x6A, 0xf3, 0x95, 0x19, 0xB4, 0xDc, 0x9a, 0x7f, 0xCc, 0xd3, 0x18, 0x21, 0x73, 0x52, 0x83,
    0x0E, 0x8A, 0xD9, 0xb4,
]);

static BLOCKS_PER_MINUTE: u8 = 30;

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

async fn get_benqi_avax_rewards_of_pool(
    web3: &Web3<Http>,
    lending_token_contract_address: &Address,
) -> Result<(f64, f64), ApiError> {
    let smart_comptroller = Web3Contract::new(
        web3.eth(),
        BENQI_COMPTROLLER_ADDRESS,
        get_benqi_comptroller_abi(),
    );
    let smart_qi_token =
        Web3Contract::new(web3.eth(), *lending_token_contract_address, get_benqi_abi());
    let reward_speed_benqi: web3::types::U256 = smart_comptroller
        .query(
            "rewardSpeeds",
            (0 as u8, *lending_token_contract_address),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
        .unwrap();
    let reward_speed_avax: web3::types::U256 = smart_comptroller
        .query(
            "rewardSpeeds",
            (1 as u8, *lending_token_contract_address),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
        .unwrap();

    let benqi_speed = reward_speed_benqi.as_u128() as f64
        / (10 as f64).powf(DIGITS_AVAX_AND_BENQI_FOR_PENDING_REWARDS as f64);
    let avax_speed = reward_speed_avax.as_u128() as f64
        / (10 as f64).powf(DIGITS_AVAX_AND_BENQI_FOR_PENDING_REWARDS as f64);

    Ok((benqi_speed, avax_speed))
}

pub async fn get_benqi_assets(
    web3: &Web3<Http>,
    wallet_address: &Address,
    contract_addresses: &Vec<Address>,
) -> Result<Vec<DefiAsset>, ApiError> {
    let mut res = Vec::new();

    let benqi_price = get_token_price(&"QI".to_string()).await?;
    let avax_price = get_token_price(&"AVAX".to_string()).await?;

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
        let price = get_benqi_token_price(&symbol).await?;

        let total_supply: web3::types::U256 = smart
            .query(
                "totalSupply",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await
            .ok()
            .unwrap();

        let (qi_block_reward, avax_block_reward) =
            get_benqi_avax_rewards_of_pool(&web3, contract_address).await?;
        let block_reward_for_pool_usd =
            qi_block_reward * benqi_price as f64 + avax_block_reward * avax_price as f64;
        let blocks_per_year = BLOCKS_PER_MINUTE as f64 * 60. * 24. * 365.;
        let pool_year_rewards_usd = block_reward_for_pool_usd as f64 * blocks_per_year;
        let pool_value_usd =
            total_supply.as_u128() as f64 / (10 as f64).powf(decimals) * price as f64;
        let pool_reward_apr = pool_year_rewards_usd / pool_value_usd * 100.; // TODO: why *100?

        let balance_usd = balance_float * exchange_rate_float;

        res.push(DefiAsset::new(
            apy_float as f32 + pool_reward_apr as f32,
            get_underlaying_name(&symbol)?,
            balance_usd as f32,
            price,
            symbol,
        ));
    }

    // Check for outstanding QI or AVAX
    // reward type 0 = QI, 1 = AVAX
    let smart = Web3Contract::new(
        web3.eth(),
        BENQI_COMPTROLLER_ADDRESS,
        get_benqi_comptroller_abi(),
    );
    let outstanding_qi: web3::types::U256 = smart
        .query(
            "rewardAccrued",            // rewardAccrued
            (0 as u8, *wallet_address), // (0 as u16, *wallet_address),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
        .unwrap();
    let outstanding_avax: web3::types::U256 = smart
        .query(
            "rewardAccrued",
            (1 as u8, *wallet_address),
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
