use crate::models::{Counter, CtxModel, FreezeModel, StaticModel};
use crate::usize;

pub type StaticPrefixModel8 = PrefixModel8<u16>;

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

impl<C: Counter> FreezeModel for PrefixModel8<C> {
    type Frozen = StaticPrefixModel8;

    fn freeze(&self) -> Self::Frozen {
        let stats = self.stats.iter().map(|c| c.p()).collect::<Vec<u16>>();
        StaticPrefixModel8 { stats, ctx: 1, lead: 1 }
    }
}

impl StaticModel for StaticPrefixModel8 {
    fn write(&self) -> Vec<u8> {
        self.stats.iter().flat_map(|&s| s.to_le_bytes()).collect()
    }

    fn read(data: &[u8]) -> Self {
        let stats = data
            .chunks_exact(2)
            .take(1 << 8)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>();
        StaticPrefixModel8 { stats, ctx: 1, lead: 1 }
    }
}

impl_prefix_analytics!(PrefixModel8, "order-0 byte tree, 8-bit context");
