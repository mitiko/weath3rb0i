use super::{counter::Counter, Model};

pub struct Order0 {
    stats: [Counter; 1 << 11],
    history: u8,
    alignment: u8,
    ctx: u16,
}

impl Order0 {
    pub fn new() -> Self {
        Self {
            stats: [Counter::new(); 1 << 11],
            history: 0,
            alignment: 0,
            ctx: 0,
        }
    }
}

impl Model for Order0 {
    fn predict(&self) -> u16 {
        self.stats[usize::from(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[usize::from(self.ctx)].update(bit);
        self.history = (self.history << 1) | bit;
        self.alignment = (self.alignment + 1) % 8;
        self.ctx = u16::from(self.alignment) << 8 | u16::from(self.history);
    }
}
