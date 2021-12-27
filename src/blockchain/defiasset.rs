use super::super::asset::Asset;
use super::super::growth::Growth;

#[derive(Clone)]
pub struct DefiAsset {
    apy: f32,
    underlaying_token_name: String,
    token_equivalent: f32,
    token_price: f32,
    description: String,
}

impl DefiAsset {
    pub fn new(
        apy: f32,
        underlaying_token_name: String,
        token_equivalent: f32,
        token_price: f32,
        description: String,
    ) -> Self {
        Self {
            apy: apy,
            underlaying_token_name: underlaying_token_name,
            token_equivalent: token_equivalent,
            token_price: token_price,
            description: description,
        }
    }
}

impl Asset for DefiAsset {
    fn get_growth(&self) -> Growth {
        Growth::new(self.apy)
    }

    fn get_name(&self) -> String {
        self.underlaying_token_name.clone()
    }

    fn get_unit_price(&self) -> f32 {
        self.token_price
    }

    fn get_units(&self) -> f32 {
        self.token_equivalent
    }

    fn get_description(&self) -> String {
        self.description.clone()
    }
}
