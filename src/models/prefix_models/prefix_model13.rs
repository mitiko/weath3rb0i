use crate::models::{Counter, CtxModel};
use crate::usize;

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
        Self { stats: vec![counter; 1 << 13], ctx: 1 << 8, lead: 1 << 8, nibble: 0 }
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
