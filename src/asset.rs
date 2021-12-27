use super::growth::Growth;

pub trait Asset: Send {
    fn get_growth(&self) -> Growth;
    fn get_name(&self) -> String;
    fn get_unit_price(&self) -> f32;
    fn get_units(&self) -> f32;
    fn get_description(&self) -> String;
}
