use std::ops::{BitOr, Shl, Index, IndexMut};

// Could be any of u8, u16, u32, u64, u128, and more!
pub trait Context: From<u8> + Into<usize> + BitOr<Output = Self> + Shl<i32, Output = Self> + Copy {}
impl<T> Context for T where T: From<u8> + Into<usize> + BitOr<Output = Self> + Shl<i32, Output = Self> + Copy {}

pub struct SmartCtx<T: Context> {
    ctx: T, // just an (unsigned) integer
    bit_id: u8,
    ctx_cache: u8
}

impl<T: Context> SmartCtx<T> {
    pub fn new(ctx: T) -> Self {
        Self { ctx, bit_id: 0, ctx_cache: 0 }
    }

    pub fn get(&self) -> SmartCtxIdx<T> {
        SmartCtxIdx(self.ctx, self.rel_idx())
    }

    pub fn get4(&self, nib: u8) -> [SmartCtxIdx<T>; 4] {[
        SmartCtxIdx(self.ctx, 0),
        SmartCtxIdx(self.ctx, 1 + (nib >> 3) as usize),
        SmartCtxIdx(self.ctx, 3 + (nib >> 2) as usize),
        SmartCtxIdx(self.ctx, 7 + (nib >> 1) as usize)
    ]}

    pub fn update(&mut self, bit: u8) {
        self.ctx_cache = (self.ctx_cache << 1) | bit;
        self.bit_id = (self.bit_id + 1) & 3;

        // TODO: Verify this is not a cmov, bc the branch predictor can easily see it's mod 4
        if self.bit_id == 0 {
            self.ctx = (self.ctx << 4) | T::from(self.ctx_cache);
            self.ctx_cache = 0;
        }
    }

    pub fn update4(&mut self, nib: u8) {
        // Do not use update and update4 interchangably
        // update4 should have the same effect as update executed on the bits of the nibble
        // but update4 is an encode onl optimization, while update is for the decoder
        assert!(self.bit_id == 0 && self.ctx_cache == 0);
        self.ctx = (self.ctx << 4) | T::from(nib);
    }

    fn rel_idx(&self) -> usize {
        // See the docs on hashslot rel_idx calculation
        usize::from((1 << self.bit_id) + (self.ctx_cache - 1))
    }
}

pub struct SmartCtxIdx<T: Context>(T, usize);

impl<T: Context, C, const N: usize, const M: usize> Index<SmartCtxIdx<T>> for [[C; M]; N] {
    type Output = C;

    fn index(&self, index: SmartCtxIdx<T>) -> &Self::Output {
        &self[index.0.into()][index.1]
    }
}

impl<T: Context, C, const N: usize, const M: usize> IndexMut<SmartCtxIdx<T>> for [[C; M]; N] {
    fn index_mut(&mut self, index: SmartCtxIdx<T>) -> &mut Self::Output {
        &mut self[index.0.into()][index.1]
    }
}

