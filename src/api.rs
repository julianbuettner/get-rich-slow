use super::account::Account;
use super::asset::Asset;
use super::blockchain::get_assets_of_ethereum_account;
use super::domainconfig::{DomainConfig, Fund};
use super::error::ApiError;
use rocket::serde::{json::json, json::Json, json::Value, Serialize};
use rocket::{Build, Rocket, State};
use serde_json::map::Map;

#[derive(Serialize)]
pub struct AssetDto {
    pub name: String,
    pub description: String,
    pub nominal_growth: f32,
    pub real_growth: f32,
    pub units: f32,
    pub unit_price: f32,
}

#[derive(Serialize)]
pub struct FundDto {
    pub name: String,
    pub icon: Option<String>,
    pub balance: f32,
    pub nominal_yearly_growth: f32,
    pub real_yearly_growth: f32,
    pub assets: Vec<AssetDto>,
}

impl FundDto {
    pub fn new(name: String, icon: Option<String>, assets: Vec<AssetDto>) -> Self {
        let eps = 0.00001; // Division by zero avoidance
        let balance: f32 = assets.iter().map(|x| x.units * x.unit_price).sum::<f32>();
        let balance_in_one_year: f32 = assets
            .iter()
            .map(|x| x.units * x.unit_price * (1. + x.nominal_growth))
            .sum();
        let real_balance_in_one_year: f32 = assets
            .iter()
            .map(|x| x.units * x.unit_price * (1. + x.real_growth))
            .sum();
        Self {
            name: name,
            icon: icon,
            balance: balance,
            nominal_yearly_growth: (balance_in_one_year + eps) / (balance + eps) - 1.,
            real_yearly_growth: (real_balance_in_one_year + eps) / (balance + eps) - 1.,
            assets: assets,
        }
    }
}

#[get("/")]
pub async fn get_overview(
    domainconfig: &State<DomainConfig>,
) -> Result<Json<Vec<FundDto>>, ApiError> {
    let mut fund_dtos = Vec::new();
    for fund in domainconfig.funds.iter() {
        let mut collected_assets = Vec::new();
        for account in fund.accounts.iter() {
            let mut assets: Vec<Box<dyn Asset>> = match account {
                Account::Etoro {
                    name: _,
                    api_key: _,
                } => todo!(),
                Account::Kraken {
                    name: _,
                    api_key: _,
                    api_secret: _,
                } => todo!(),
                Account::Ethereum(eth_account) => {
                    get_assets_of_ethereum_account(domainconfig, eth_account)
                        .await?
                        .iter_mut()
                        .map(|x| Box::new(x.clone()) as Box<dyn Asset>)
                        .collect::<Vec<Box<dyn Asset>>>()
                }
            };
            collected_assets.append(&mut assets);
        }
        let asset_dtos = collected_assets
            .iter()
            .map(|a| AssetDto {
                name: a.get_name(),
                nominal_growth: a.get_growth().get_nominal_growth(),
                real_growth: a.get_growth().get_real_growth(),
                units: a.get_units(),
                unit_price: a.get_unit_price(),
                description: a.get_description(),
            })
            .collect();
        fund_dtos.push(FundDto::new(
            fund.name.clone(),
            fund.icon.clone(),
            asset_dtos,
        ));
    }

    Ok(Json(fund_dtos))
}

#[get("/block")]
pub async fn get_block(domainconfig: &State<DomainConfig>) -> Result<Value, ApiError> {
    let mut map = Map::new();

    let tasks: Vec<(String, _)> = domainconfig
        .eth_nodes
        .iter()
        .map(|node| {
            (
                node.chain.to_str().to_string(),
                tokio::spawn(node.web3.eth().block_number()),
            )
        })
        .collect();

    for (name, task) in tasks {
        let result = task.await.unwrap();
        map.insert(
            name,
            match result {
                Ok(v) => json!(v.as_u64()),
                Err(e) => json!(e.to_string()),
            },
        );
    }

    Ok(Value::Object(map))
}

pub fn get_rocket_build(domainconfig: DomainConfig) -> Rocket<Build> {
    rocket::build()
        .manage(domainconfig)
        .mount("/", routes![get_overview, get_block,])
}
