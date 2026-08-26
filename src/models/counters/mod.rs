pub mod linear_counter4;
pub mod geometric;
pub mod fsm0;

use crate::{u16, Analytics};

pub use linear_counter4::*;
pub use geometric::*;
pub use fsm0::*;

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
