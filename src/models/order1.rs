use super::{counter::Counter, Model};
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

impl Model for Order1 {
    fn predict(&self) -> u16 {
        self.stats[usize!(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        let ctx = usize!(self.ctx);
        self.stats[ctx].update(bit);
        self.history = (self.history << 1) | u16::from(bit);
        self.alignment = (self.alignment + 1) % 8;
        self.ctx = u32::from(self.alignment) << 16 | u32::from(self.history);
    }
}
