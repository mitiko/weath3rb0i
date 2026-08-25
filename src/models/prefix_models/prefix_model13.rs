use crate::models::{Counter, CtxModel, FreezeModel, StaticModel};
use crate::usize;

pub type StaticPrefixModel13 = PrefixModel13<u16>;

/// order-2 nibble tree, 13-bit context
/// ```text
/// 0_0001_abcd_efgh
/// 0_001a_bcde_fghi
/// 0_01ab_cdef_ghij
/// 0_1abc_defg_hijk
/// 1_0001_efgh_ijkl
/// 1_001e_fghi_jklm
/// 1_01ef_ghij_klmn
/// 1_1efg_hijk_lmno
/// ```
pub struct PrefixModel13<C: Counter> {
    stats: Vec<C>,
    ctx: usize,
    lead: usize,
    nibble: usize,
}

impl<C: Counter> PrefixModel13<C> {
    pub fn new(counter: C) -> Self {
        Self {
            stats: vec![counter; 1 << 13],
            ctx: 1 << 8,
            lead: 1 << 8,
            nibble: 0,
        }
    }
}

impl<C: Counter> CtxModel for PrefixModel13<C> {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn set_ctx(&mut self, hash: u32) {
        self.lead <<= 1;
        if self.lead == 1 << 12 {
            self.lead = 1 << 8;
            self.nibble ^= 1 << 12;
        }
        let mask = self.lead - 1;
        self.ctx = self.nibble | self.lead | (usize!(hash) & mask);
    }
}

impl<C: Counter> PrefixModel13<C> {
    pub fn freeze(&self) -> StaticPrefixModel13 {
        let stats = self.stats.iter().map(|c| c.p()).collect::<Vec<u16>>();
        StaticPrefixModel13 { stats, ctx: 1 << 8, lead: 1 << 8, nibble: 0 }
    }
}

impl<C: Counter> FreezeModel for PrefixModel13<C> {
    type Frozen = StaticPrefixModel13;

    fn freeze(&self) -> Self::Frozen {
        let stats = self.stats.iter().map(|c| c.p()).collect::<Vec<u16>>();
        StaticPrefixModel13 { stats, ctx: 1 << 8, lead: 1 << 8, nibble: 0 }
    }
}

impl StaticModel for StaticPrefixModel13 {
    fn write(&self) -> Vec<u8> {
        self.stats.iter().flat_map(|&s| s.to_le_bytes()).collect()
    }

    fn read(data: &[u8]) -> Self {
        let stats = data
            .chunks_exact(2)
            .take(1 << 13)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>();
        StaticPrefixModel13 { stats, ctx: 1 << 8, lead: 1 << 8, nibble: 0 }
    }
}

impl_prefix_analytics!(PrefixModel13, "order-2 nibble tree, 13-bit context");
