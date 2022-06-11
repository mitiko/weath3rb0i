pub mod order0;
pub mod order1;
pub mod counter;

pub use crate::state_table::*;
pub use crate::smart_context::*;
pub use self::{order0::*, order1::*, counter::*};

// TODO: Rename to PrefixModel and use a context as parameter to predictions, no updates?
pub trait Model {
    fn predict(&self) -> u16;
    fn predict4(&self, nib: u8) -> [u16; 4];

    fn update(&mut self, bit: u8);
    fn update4(&mut self, nib: u8);
}
