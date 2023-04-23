use super::{counter::Counter, SharedCtx, SharedModel, SmartCtx};

pub struct Order1 {
    stats: [[Counter; 15]; 1 << 16],
}

impl Order1 {
    pub fn init() -> Self {
        Self { stats: [[Counter::new(); 15]; 1 << 16] }
    }
}

const MASK: u64 = u16::MAX as u64;

impl SharedModel<SmartCtx> for Order1 {
    fn predict(&self, ctx: &SmartCtx) -> u16 {
        self.stats[ctx.get(MASK)].p()
    }

    fn update(&mut self, ctx: &SmartCtx, bit: u8) {
        self.stats[ctx.get(MASK)].update(bit);
    }

    fn predict4(&self, ctx: &SmartCtx, nib: u8) -> [u16; 4] {
        let [idx1, idx2, idx3, idx4] = ctx.get4(MASK, nib);
        [
            self.stats[idx1].p(),
            self.stats[idx2].p(),
            self.stats[idx3].p(),
            self.stats[idx4].p(),
        ]
    }

    fn update4(&mut self, ctx: &SmartCtx, nib: u8) {
        let [idx1, idx2, idx3, idx4] = ctx.get4(MASK, nib);
        self.stats[idx1].update(nib >> 3);
        self.stats[idx2].update((nib >> 2) & 1);
        self.stats[idx3].update((nib >> 1) & 1);
        self.stats[idx4].update(nib & 1);
    }
}
