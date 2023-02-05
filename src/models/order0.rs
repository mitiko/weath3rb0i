use super::{Model, SharedCtx, SmartCtx, counter::Counter};

pub struct Order0 {
    stats: [[Counter; 15]; 256]
}

impl Order0 {
    pub fn init() -> Self {
        Self { stats: [[Counter::new(); 15]; 256] }
    }
}

const MASK: u64 = u8::MAX as u64;

impl Model<SmartCtx> for Order0 {
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
            self.stats[idx4].p()
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
