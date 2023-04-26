use super::{counter::Counter, Model};

pub struct Order0 {
    stats: [Counter; 1 << 11],
    history: u8,
    alignment: u8,
}

impl Order0 {
    pub fn new() -> Self {
        Self {
            stats: [Counter::new(); 1 << 11],
            history: 0,
            alignment: 0,
        }
    }
}

impl Model for Order0 {
    fn predict(&self) -> u16 {
        let ctx = usize::from(self.history) << 3 | usize::from(self.alignment);
        self.stats[ctx].p()
    }

    fn update(&mut self, bit: u8) {
        let ctx = usize::from(self.history) << 3 | usize::from(self.alignment);
        self.stats[ctx].update(bit);
        self.history = (self.history << 1) | bit;
        self.alignment = (self.alignment + 1) % 8;
    }
}
