use super::super::cryptoprice::get_token_price;
use super::super::error::ApiError;
use super::abi::get_venus_abi;
use super::defiasset::DefiAsset;
use regex::Regex;
use web3::contract::Contract as Web3Contract;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

fn get_underlaying_name(vtoken: &String) -> Result<String, ApiError> {
    let pattern = Regex::new(r"(v|x)(?P<underlaying>[A-Z]{3,4})").unwrap();
    let capture = pattern.captures(&vtoken);
    if capture.is_none() {
        return Err(ApiError::new(&format!("No valid Venus Token: {}", vtoken)));
    }
    Ok(capture.unwrap()["underlaying"].to_string())
}

async fn get_vtoken_price(vtoken: &String) -> Result<f32, ApiError> {
    get_token_price(&get_underlaying_name(vtoken)?).await
}

pub async fn get_venus_assets(
    web3: &Web3<Http>,
    wallet_address: &Address,
    contract_addresses: &Vec<Address>,
) -> Result<Vec<DefiAsset>, ApiError> {
    let mut res = Vec::new();

    for contract_address in contract_addresses.iter() {
        let smart = Web3Contract::new(web3.eth(), *contract_address, get_venus_abi());

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

        let exchange_rate: web3::types::U256 = smart
            .query(
                "exchangeRateCurrent",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await?;
        let exchange_rate = exchange_rate.as_u128();
        let exchange_rate_float = exchange_rate as f64 * (10.0 as f64).powf(-28.);

        let balance = balance.as_u128();
        let balance_float = balance as f64 / (10 as f64).powf(decimals);

        let symbol: String = smart
            .query("symbol", (), None, web3::contract::Options::default(), None)
            .await?;

        let rate_per_block: web3::types::U256 = smart
            .query(
                "supplyRatePerBlock",
                (),
                None,
                web3::contract::Options::default(),
                None,
            )
            .await?;

        let yield_as_float = rate_per_block.as_u128() as f64 * (10. as f64).powf(-18.);
        let apy = (yield_as_float * 20. * 60. * 24. + 1.).powf(365.) - 1.;

        res.push(DefiAsset::new(
            apy as f32,
            get_underlaying_name(&symbol)?,
            balance_float as f32 * exchange_rate_float as f32,
            get_vtoken_price(&symbol).await?,
            symbol,
        ));
    }

    Ok(res)
}
