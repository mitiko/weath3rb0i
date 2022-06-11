use super::{Model, counter::Counter, SmartCtx};

pub struct Order0 {
    ctx: SmartCtx<u8>,
    stats: [[Counter; 15]; 256]
}

impl Order0 {
    pub fn init() -> Self {
        Self {
            ctx: SmartCtx::new(0),
            stats: [[Counter::new(); 15]; 256]
        }
    }
}

impl Model for Order0 {
    fn predict(&self) -> u16 {
        self.stats[self.ctx.get()].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[self.ctx.get()].update(bit);
        self.ctx.update(bit);
    }

    fn predict4(&self, nib: u8) -> [u16; 4] {
        let [idx1, idx2, idx3, idx4] = self.ctx.get4(nib);
        [
            self.stats[idx1].p(),
            self.stats[idx2].p(),
            self.stats[idx3].p(),
            self.stats[idx4].p()
        ]
    }

    fn update4(&mut self, nib: u8) {
        let [idx1, idx2, idx3, idx4] = self.ctx.get4(nib);
        self.stats[idx1].update(nib >> 3);
        self.stats[idx2].update((nib >> 2) & 1);
        self.stats[idx3].update((nib >> 1) & 1);
        self.stats[idx4].update(nib & 1);
        self.ctx.update4(nib);
    }
}
