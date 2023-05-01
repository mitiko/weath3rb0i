use super::{counter::Counter, Model};
use crate::history::History;

pub struct Order1Entropy {
    stats: Vec<Counter>,
    history: History,
    ctx: u32,
}

impl Order1Entropy {
    pub fn new() -> Self {
        Self {
            stats: vec![Counter::new(); 1 << 19],
            history: History::new(),
            ctx: 0,
        }
    }
}

impl Model for Order1Entropy {
    fn predict(&self) -> u16 {
        let ctx = usize::try_from(self.ctx).unwrap();
        self.stats[ctx].p()
    }

    fn update(&mut self, bit: u8) {
        let ctx = usize::try_from(self.ctx).unwrap();
        self.stats[ctx].update(bit);
        self.history.update(bit);
        self.ctx = self.history.hash();
    }
}
