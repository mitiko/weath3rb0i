pub mod ac_history;
pub mod ac_history_skip;
pub mod huff_history;

pub use self::{ac_history::*, ac_history_skip::*, huff_history::*};

pub trait History {
    fn update(&mut self, bit: u8);
    fn hash(&mut self, max_bits: u8) -> u32;
}

impl History for u32 {
    fn update(&mut self, bit: u8) {
        *self = (*self << 1) | u32::from(bit);
    }

    fn hash(&mut self, _max_bits: u8) -> u32 {
        *self
    }
}
