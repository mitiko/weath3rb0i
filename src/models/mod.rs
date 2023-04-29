pub mod counter;
pub mod order0;
pub mod order0entropy;
pub mod stationary;

pub use self::{counter::*, order0::*, order0entropy::*};
pub use crate::state_table::*;

pub trait Model {
    fn predict(&self) -> u16;
    fn update(&mut self, bit: u8);
}

pub trait StationaryModel {
    fn predict(&mut self) -> u16;
}
