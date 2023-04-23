pub mod counter;
mod nib_tree;
pub mod order0;
pub mod order1;
pub mod tiny_order0;

pub use self::{counter::*, order0::*, order1::*, tiny_order0::*};
pub use crate::smart_context::*;
pub use crate::state_table::*;

// TODO: Rename to PrefixModel and use a context as parameter to predictions, no updates?
pub trait SharedModel<Ctx: SharedCtx> {
    fn predict(&self, ctx: &Ctx) -> u16;
    fn predict4(&self, ctx: &Ctx, nib: u8) -> [u16; 4];

    fn update(&mut self, ctx: &Ctx, bit: u8);
    fn update4(&mut self, ctx: &Ctx, nib: u8) {
        self.update(ctx, nib >> 3);
        self.update(ctx, (nib >> 2) & 1);
        self.update(ctx, (nib >> 1) & 1);
        self.update(ctx, nib & 1);
    }
}

pub trait Model {
    fn predict(&self) -> u16;
    fn update(&mut self, bit: u8);
}

pub trait Model4: Model {
    fn predict4(&self, nib: u8) -> [u16; 4];
    fn update4(&mut self, nib: u8);
}

pub trait SharedCtx {
    type Idx: Sized;

    fn new() -> Self;

    fn get(&self, mask: u64) -> Self::Idx;
    fn get4(&self, mask: u64, nib: u8) -> [Self::Idx; 4];

    fn update(&mut self, bit: u8);
    fn update4(&mut self, nib: u8) {
        self.update(nib >> 3);
        self.update((nib >> 2) & 1);
        self.update((nib >> 1) & 1);
        self.update(nib & 1);
    }
}
