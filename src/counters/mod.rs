mod bit_counter;
mod record_keeper;

use crate::models::Model;

pub use self::{bit_counter::*, record_keeper::*};

pub trait Counter : Model + Copy {}

pub trait RecordCounter : Copy {
    fn new() -> Self;
    fn predict(&self, pos: u16) -> u16;
    fn update(&mut self, pos: u16, bit: u8);
}
