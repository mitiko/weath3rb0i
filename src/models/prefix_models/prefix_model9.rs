use crate::models::{Counter, CtxModel, FreezeModel, StaticModel};
use crate::usize;

pub type StaticPrefixModel9 = PrefixModel9<u16>;

/// order-0 nibble tree, 9-bit context
/// ```text
/// 0_0001_abcd
/// 0_001a_bcde
/// 0_01ab_cdef
/// 0_1abc_defg
/// 1_0001_efgh
/// 1_001e_fghi
/// 1_01ef_ghij
/// 1_1efg_hijk
/// ```
pub struct PrefixModel9<C: Counter> {
    stats: Vec<C>,
    ctx: usize,
    lead: usize,
    nibble: usize,
}

impl<C: Counter> PrefixModel9<C> {
    pub fn new(counter: C) -> Self {
        Self {
            stats: vec![counter; 1 << 9],
            ctx: 1 << 4,
            lead: 1 << 4,
            nibble: 0,
        }
    }
}

impl<C: Counter> CtxModel for PrefixModel9<C> {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn set_ctx(&mut self, hash: u32) {
        self.lead <<= 1;
        if self.lead == 1 << 8 {
            self.lead = 1 << 4;
            self.nibble ^= 1 << 8;
        }
        let mask = self.lead - 1;
        self.ctx = self.nibble | self.lead | (usize!(hash) & mask);
    }
}

impl<C: Counter> FreezeModel for PrefixModel9<C> {
    type Frozen = StaticPrefixModel9;

    fn freeze(&self) -> Self::Frozen {
        let stats = self.stats.iter().map(|c| c.p()).collect::<Vec<u16>>();
        StaticPrefixModel9 { stats, ctx: 1 << 4, lead: 1 << 4, nibble: 0 }
    }
}

impl StaticModel for StaticPrefixModel9 {
    fn write(&self) -> Vec<u8> {
        self.stats.iter().flat_map(|&s| s.to_le_bytes()).collect()
    }

    fn read(data: &[u8]) -> Self {
        let stats = data
            .chunks_exact(2)
            .take(1 << 9)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>();
        StaticPrefixModel9 { stats, ctx: 1 << 4, lead: 1 << 4, nibble: 0 }
    }
}
