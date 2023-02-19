mod order0;
mod record_model;
mod nib_tree;

use crate::counters::{BitCounter, ExactRecordKeeper};

pub type Order0 = self::order0::Order0<BitCounter>;
pub type RecordModel = self::record_model::RecordModel<ExactRecordKeeper>;

pub trait Model {
    fn new() -> Self;
    fn predict(&self) -> u16;
    fn update(&mut self, bit: u8);
}

pub trait Model4 : Model {
    fn predict4(&self, nib: u8) -> [u16; 4];
    fn update4(&mut self, nib: u8);
}
