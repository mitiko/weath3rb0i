use crate::entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder};
use crate::u8;
use std::collections::HashMap;

pub mod raw_history;
pub mod ac_history;
pub use raw_history::*;
pub use ac_history::*;

pub trait History {
    fn update(&mut self, bit: u8);
    fn hash(&mut self) -> u32;
}
