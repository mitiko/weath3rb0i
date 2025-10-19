use super::{counter::Counter, AdaptiveModel};
use crate::usize;

pub struct Order1 {
    stats: Vec<Counter>,
    history: u16,
    alignment: u8,
    ctx: u32,
}

impl Order1 {
    pub fn new() -> Self {
        Self {
            stats: vec![Counter::new(); 1 << 19],
            history: 0,
            alignment: 0,
            ctx: 0,
        }
    }
}

impl AdaptiveModel for Order1 {
    fn predict(&self) -> u16 {
        self.stats[usize!(self.ctx)].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[usize!(self.ctx)].update(bit);
    }

    fn update(&mut self, bit: u8) {
        self.history = (self.history << 1) | u16::from(bit);
        self.alignment = (self.alignment + 1) % 8;
        self.ctx = u32::from(self.alignment) << 16 | u32::from(self.history);
    }
}
