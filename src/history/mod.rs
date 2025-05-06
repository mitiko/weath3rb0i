pub mod ac_history;
pub mod ac_history_cached;
pub mod huff_history;
pub mod raw_history;

pub use self::{ac_history::*, ac_history_cached::*, huff_history::*, raw_history::*};

pub trait History {
    fn update(&mut self, bit: u8);
    fn hash(&mut self) -> u32;
}
