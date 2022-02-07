use crate::state_table::StateTable;

pub mod order0;
pub mod counter;

// TODO: Rename to PrefixModel and use a context as parameter to predictions, no updates?
pub trait Model<'a> {
    fn init(state_table: &'a StateTable) -> Self;

    fn predict(&self) -> u16;
    fn predict4(&self, nib: u8) -> [u16; 4];

    fn update(&mut self, bit: u8);
    fn update4(&mut self, nib: u8);
    // fn update4(&mut self, nib: u8) {
    //     self.update( nib >> 3);
    //     self.update((nib >> 2) & 1);
    //     self.update((nib >> 1) & 1);
    //     self.update( nib       & 1);
    // }
}
