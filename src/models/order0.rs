use super::{Model, counter::Counter};

pub struct Order0 {
    ctx: u8,
    ctx_cache: u8,
    bit_id: u8,
    stats: [[Counter; 15]; 256]
}

impl Order0 {
    pub fn init() -> Self { Self { ctx: 0, ctx_cache: 0, bit_id: 0, stats: [[Counter::new(); 15]; 256] } }
}

impl Model for Order0 {
    fn predict(&self) -> u16 {
        let idx = (7 >> (3 - self.bit_id)) + self.ctx_cache as usize;
        self.stats[self.ctx as usize][idx].p()
    }

    fn update(&mut self, bit: u8) {
        let idx = (7 >> (3 - self.bit_id)) + self.ctx_cache as usize;
        self.stats[self.ctx as usize][idx].update(bit);

        self.ctx_cache = (self.ctx_cache << 1) | bit;
        self.bit_id = (self.bit_id + 1) & 3;

        // TODO: Verify this is not a cmov, bc the branch predictor can easily see it's mod 4
        if self.bit_id == 0 {
            self.ctx = (self.ctx << 4) | self.ctx_cache;
            self.ctx_cache = 0;
        }
    }

    fn predict4(&self, nib: u8) -> [u16; 4] {[
        self.stats[self.ctx as usize][0].p(),
        self.stats[self.ctx as usize][1 + (nib >> 3) as usize].p(),
        self.stats[self.ctx as usize][3 + (nib >> 2) as usize].p(),
        self.stats[self.ctx as usize][7 + (nib >> 1) as usize].p()
    ]}

    fn update4(&mut self, nib: u8) {
        self.stats[self.ctx as usize][0]                      .update( nib >> 3);
        self.stats[self.ctx as usize][1 + (nib >> 3) as usize].update((nib >> 2) & 1);
        self.stats[self.ctx as usize][3 + (nib >> 2) as usize].update((nib >> 1) & 1);
        self.stats[self.ctx as usize][7 + (nib >> 1) as usize].update( nib       & 1);
        self.ctx = (self.ctx << 4) | nib;
    }
}
