use super::growth::Growth;

#[derive(Clone)]
pub struct GenericAsset {
    apy: f32,
    trading_symbol: String,
    description: String,
    units: f32,
    unit_price: f32,
}

impl GenericAsset {
    pub fn new(
        apy: f32,
        trading_symbol: String,
        description: String,
        units: f32,
        unit_price: f32,
    ) -> Self {
        Self {
            apy: apy,
            trading_symbol: trading_symbol,
            description: description,
            units: units,
            unit_price: unit_price,
        }
    }
}

pub trait Asset: Send {
    fn get_growth(&self) -> Growth;
    fn get_name(&self) -> String;
    fn get_unit_price(&self) -> f32;
    fn get_units(&self) -> f32;
    fn get_description(&self) -> String;
}

impl Asset for GenericAsset {
    fn get_growth(&self) -> Growth {
        Growth::new(self.apy)
    }

    fn get_name(&self) -> String {
        self.trading_symbol.clone()
    }

    fn get_unit_price(&self) -> f32 {
        self.unit_price
    }

    fn get_units(&self) -> f32 {
        self.units
    }

    fn get_description(&self) -> String {
        self.description.clone()
    }
}
