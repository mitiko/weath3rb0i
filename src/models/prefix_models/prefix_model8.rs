use crate::models::{Counter, CtxModel};
use crate::usize;

/// order-0 byte tree, 8-bit context
/// ```text
/// 0000_0001
/// 0000_001a
/// 0000_01ab
/// 0000_1abc
/// 0001_abcd
/// 001a_bcde
/// 01ab_cdef
/// 1abc_defg
/// ```
pub struct PrefixModel8<C: Counter> {
    stats: Vec<C>,
    ctx: usize,
    lead: usize,
}

impl<C: Counter> PrefixModel8<C> {
    pub fn new(counter: C) -> Self {
        Self { stats: vec![counter; 1 << 8], ctx: 1, lead: 1 }
    }
}

impl<C: Counter> CtxModel for PrefixModel8<C> {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn set_ctx(&mut self, hash: u32) {
        self.lead <<= 1;
        if self.lead == 1 << 8 {
            self.lead = 1;
        }
        let mask = self.lead - 1;
        self.ctx = self.lead | (usize!(hash) & mask);
    }
}
