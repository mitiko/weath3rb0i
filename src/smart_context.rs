use std::ops::{Index, IndexMut};

use crate::models::SharedCtx;

#[derive(Clone, Copy, Debug, Default)]
pub struct SmartCtx {
    ctx: u64,
    bit_id: u8,
    ctx_cache: u8
}

impl SmartCtx {
    /// See the docs on hashslot rel_idx calculation
    fn rel_idx(&self) -> usize {
        // usize::from((1u8 << self.bit_id).wrapping_add(self.ctx_cache.wrapping_sub(1)))
        usize::from((1u8 << self.bit_id) - 1 + self.ctx_cache)
    }
}

impl SharedCtx for SmartCtx {
    type Idx = SmartCtxIdx;

    fn new() -> Self { Self::default() }

    fn get(&self, mask: u64) -> SmartCtxIdx {
        SmartCtxIdx((self.ctx & mask).try_into().unwrap(), self.rel_idx())
    }

    #[cfg(feature = "nib-ops")]
    fn get4(&self, mask: u64, nib: u8) -> [SmartCtxIdx; 4] {
        let ctx = (self.ctx & mask).try_into().unwrap();
        [
            SmartCtxIdx(ctx, 0),
            SmartCtxIdx(ctx, usize::from(1 + (nib >> 3))),
            SmartCtxIdx(ctx, usize::from(3 + (nib >> 2))),
            SmartCtxIdx(ctx, usize::from(7 + (nib >> 1)))
        ]
    }

    fn update(&mut self, bit: u8) {
        self.ctx_cache = (self.ctx_cache << 1) | bit;
        self.bit_id = (self.bit_id + 1) & 3;

        if self.bit_id == 0 {
            self.ctx = (self.ctx << 4) | u64::from(self.ctx_cache);
            self.ctx_cache = 0;
        }
    }

    /// Do not use `update` and `update4` interchangably
    /// `update4` should have the same effect as update executed on the bits of the nibble
    /// but `update4` is an encode only optimization, while `update` is for the decoder
    #[cfg(feature = "nib-ops")]
    fn update4(&mut self, nib: u8) {
        debug_assert!(self.bit_id == 0 && self.ctx_cache == 0);
        self.ctx = (self.ctx << 4) | u64::from(nib);
    }
}

#[derive(Clone, Copy)]
pub struct SmartCtxIdx(usize, usize);

impl<C, const N: usize, const M: usize> Index<SmartCtxIdx> for [[C; M]; N] {
    type Output = C;

    fn index(&self, index: SmartCtxIdx) -> &Self::Output {
        &self[index.0][index.1]
    }
}

impl<C, const N: usize, const M: usize> IndexMut<SmartCtxIdx> for [[C; M]; N] {
    fn index_mut(&mut self, index: SmartCtxIdx) -> &mut Self::Output {
        &mut self[index.0][index.1]
    }
}

