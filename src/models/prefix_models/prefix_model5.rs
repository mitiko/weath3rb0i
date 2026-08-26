use crate::{models::*, usize};

pub type StaticPrefixModel5 = PrefixModel5<u16>;

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
    nibble: usize,
}

impl<C: Counter> PrefixModel5<C> {
    pub fn new(counter: C) -> Self {
        Self { stats: vec![counter; 1 << 5], ctx: 1, nibble: 0 }
    }
}

impl<C: Counter> AdaptiveModel for PrefixModel5<C> {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn update(&mut self, bit: u8) {
        self.ctx = self.ctx << 1 | usize!(bit);
        if self.ctx ^ self.nibble >= 1 << 4 {
            self.nibble ^= 1 << 4;
            self.ctx = self.nibble | 1;
        }
    }
}

impl<C: Counter> FreezeModel for PrefixModel5<C> {
    type Frozen = StaticPrefixModel5;

    fn freeze(&self) -> Self::Frozen {
        let stats = self.stats.iter().map(|c| c.p()).collect::<Vec<u16>>();
        StaticPrefixModel5 { stats, ctx: 1, nibble: 0 }
    }
}

impl SerializableModel for StaticPrefixModel5 {
    fn write(&self) -> Vec<u8> {
        self.stats.iter().flat_map(|&s| s.to_le_bytes()).collect()
    }

    fn read(data: &[u8]) -> Self {
        let stats = data
            .chunks_exact(2)
            .take(1 << 5)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect::<Vec<u16>>();
        StaticPrefixModel5 { stats, ctx: 1, nibble: 0 }
    }
}

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
pub struct CtxPrefixModel5<C: Counter> {
    stats: Vec<C>,
    ctx: usize,
    lead: usize,
    nibble: usize,
}

impl<C: Counter> CtxPrefixModel5<C> {
    pub fn new(counter: C) -> Self {
        Self {
            stats: vec![counter; 1 << 5],
            ctx: 1,
            lead: 1,
            nibble: 0,
        }
    }
}

impl<C: Counter> CtxModel for CtxPrefixModel5<C> {
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
