use crate::models::{Counter, CtxModel, FreezeModel, StaticModel};
use crate::usize;

pub type StaticPrefixModel16 = PrefixModel16<u16>;

/// order-1 byte tree, 16-bit context
/// ```text
/// 0000_0001_abcd_efgh
/// 0000_001a_bcde_fghi
/// 0000_01ab_cdef_ghij
/// 0000_1abc_defg_hijk
/// 0001_abcd_efgh_ijkl
/// 001a_bcde_fghi_jklm
/// 01ab_cdef_ghij_klmn
/// 1abc_defg_hijk_lmno
/// ```
#[derive(Clone)]
pub struct PrefixModel16<C: Counter> {
    stats: Vec<C>,
    ctx: usize,
    lead: usize,
}

impl<C: Counter> PrefixModel16<C> {
    pub fn new(counter: C) -> Self {
        Self {
            stats: vec![counter; 1 << 16],
            ctx: 1 << 8,
            lead: 1 << 8,
        }
    }
}

impl<C: Counter> CtxModel for PrefixModel16<C> {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn set_ctx(&mut self, hash: u32) {
        self.lead <<= 1;
        if self.lead == 1 << 16 {
            self.lead = 1 << 8;
        }
        let mask = self.lead - 1;
        self.ctx = self.lead | (usize!(hash) & mask);
    }
}

impl<C: Counter> FreezeModel for PrefixModel16<C> {
    type Frozen = StaticPrefixModel16;

    fn freeze(&self) -> StaticPrefixModel16 {
        let stats = self.stats.iter().map(|c| c.p()).collect::<Vec<u16>>();
        StaticPrefixModel16 { stats, ctx: 1 << 8, lead: 1 << 8 }
    }
}

impl StaticModel for StaticPrefixModel16 {
    fn write(&self) -> Vec<u8> {
        self.stats.iter().flat_map(|&s| s.to_le_bytes()).collect()
    }

    fn read(data: &[u8]) -> Self {
        let stats = data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take(1 << 16)
            .collect::<Vec<u16>>();
        StaticPrefixModel16 { stats, ctx: 1 << 8, lead: 1 << 8 }
    }
}
