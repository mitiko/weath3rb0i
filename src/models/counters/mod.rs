pub mod fsm0;
pub mod geometric;
pub mod linear_counter4;

use crate::{u16, Analytics};

pub use fsm0::*;
pub use geometric::*;
pub use linear_counter4::*;

pub trait Counter: Sized + Clone {
    fn p(&self) -> u16;
    fn update(&mut self, bit: u8);
}

/// static probability
impl Counter for u16 {
    fn p(&self) -> u16 {
        *self
    }

    fn update(&mut self, _bit: u8) {}
}
