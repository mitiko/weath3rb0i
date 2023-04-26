pub mod counter;
pub mod order0;

pub use self::{counter::*, order0::*};
pub use crate::state_table::*;

pub trait Model {
    fn predict(&self) -> u16;
    fn update(&mut self, bit: u8);
}
