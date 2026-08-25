pub mod ac_history;
pub mod huff_history;

pub use self::{ac_history::*, huff_history::*};

pub trait History {
    fn update(&mut self, bit: u8);
    fn hash(&mut self) -> u32;
}

impl History for u32 {
    fn update(&mut self, bit: u8) {
        *self = (*self << 1) | u32::from(bit);
    }

    fn hash(&mut self) -> u32 {
        *self
    }
}
