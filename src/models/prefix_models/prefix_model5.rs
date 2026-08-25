use crate::models::{Counter, CtxModel};
use crate::usize;

/// order-0 nibble tree, 5-bit context
/// ```text
/// 0000_0001
/// 0000_001a
/// 0000_01ab
/// 0000_1abc
/// 0001_0001
/// 0001_001a
/// 0001_01ab
/// 0001_1abc
/// ```
pub struct PrefixModel5<C: Counter> {
    stats: Vec<C>,
    ctx: usize,
    lead: usize,
    nibble: usize,
}

impl<C: Counter> PrefixModel5<C> {
    pub fn new(counter: C) -> Self {
        Self { stats: vec![counter; 1 << 5], ctx: 1, lead: 1, nibble: 0 }
    }
}

impl<C: Counter> CtxModel for PrefixModel5<C> {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn set_ctx(&mut self, hash: u32) {
        self.lead <<= 1;
        if self.lead == 1 << 4 {
            self.lead = 1;
            self.nibble ^= 1 << 4;
        }
        let mask = self.lead - 1;
        self.ctx = self.nibble | self.lead | (usize!(hash) & mask);
    }
}
