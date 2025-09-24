use crate::models::Order0;

use super::{counter::Counter, Model};

pub struct StaticOrder0 {
    stats: [Counter; 1 << 11],
    history: u8,
    alignment: u8,
    ctx: u16,
}

impl StaticOrder0 {
    pub fn new(model: Order0) -> Self {
        Self {
            stats: model.stats,
            history: 0,
            alignment: 0,
            ctx: 0,
        }
    }
}

impl Model for StaticOrder0 {
    fn predict(&self) -> u16 {
        self.stats[usize::from(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        self.history = (self.history << 1) | bit;
        self.alignment = (self.alignment + 1) & 7;
        self.ctx = u16::from(self.alignment) << 8 | u16::from(self.history);
    }
}
