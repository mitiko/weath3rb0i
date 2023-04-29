use super::{counter::Counter, Model};
use crate::history::History;

pub struct Order0Entropy {
    stats: [Counter; 1 << 11],
    history: History,
    ctx: u16,
}

impl Order0Entropy {
    pub fn new() -> Self {
        Self {
            stats: [Counter::new(); 1 << 11],
            history: History::new(),
            ctx: 0,
        }
    }
}

impl Model for Order0Entropy {
    fn predict(&self) -> u16 {
        self.stats[usize::from(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[usize::from(self.ctx)].update(bit);
        self.history.update(bit);
        self.ctx = self.history.hash();
    }
}
