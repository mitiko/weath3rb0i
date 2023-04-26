use super::{counter::Counter, Model};

pub struct Order0Tiny {
    stats: [Counter; 1 << 11],
    history: u8,
    mask: u8,
}

impl Order0Tiny {
    pub fn new() -> Self {
        Self {
            stats: [Counter::new(); 1 << 11],
            history: 0,
            mask: 0,
        }
    }
}

impl Model for Order0Tiny {
    fn predict(&self) -> u16 {
        let ctx = usize::from(self.mask + (self.history & self.mask));
        self.stats[ctx].p()
    }

    fn update(&mut self, bit: u8) {
        let ctx = usize::from(self.mask + (self.history & self.mask));
        self.stats[ctx].update(bit);
        self.history = (self.history << 1) | bit;
        // 15 -> 31 -> 63 -> 127 -> 255 -> 511 -> 1023 -> 15 (mask)
        // 0 -> 16 -> 48 -> 112 -> 240 -> 496 -> 1008 -> 0 (reserved)
        // 15 -> 31 -> 63 -> 127 ->255 -> 255 -> 255 -> 255 (mask)
        // 0 -> 16 -> 48 -> 112 -> 240 -> 496 -> 752 -> 1008 (reserved)
        // 0 -> 1 -> 3 -> 7 -> 15 -> 31 -> 63 -> 127 -> 0
        self.mask = ((self.mask + 1) << 1).saturating_sub(1);
    }
}
