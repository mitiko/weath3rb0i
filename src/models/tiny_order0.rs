use super::{Model, SharedCtx, SmartCtx, counter::Counter};
use super::{StateTable, naive::NaiveStateTable};

pub struct TinyOrder0 {
    stats: [[u16; 15]; 256]
}

impl TinyOrder0 {
    pub fn init() -> Self {
        Self { stats: [[0; 15]; 256] }
    }
}

const MASK: u64 = u8::MAX as u64;

impl Model<SmartCtx> for TinyOrder0 {
    fn predict(&self, ctx: &SmartCtx) -> u16 {
        let state = self.stats[ctx.get(MASK)];
        NaiveStateTable::p(state)
    }
    
    fn update(&mut self, ctx: &SmartCtx, bit: u8) {
        let ctx = ctx.get(MASK);
        let state = self.stats[ctx];
        self.stats[ctx] = NaiveStateTable::next(state, bit);
    }

    fn predict4(&self, ctx: &SmartCtx, nib: u8) -> [u16; 4] {
        let [idx1, idx2, idx3, idx4] = ctx.get4(MASK, nib);
        let states = [self.stats[idx1], self.stats[idx2], self.stats[idx3], self.stats[idx4]];
        NaiveStateTable::p4(states)
    }
    
    fn update4(&mut self, ctx: &SmartCtx, nib: u8) {
        let [idx1, idx2, idx3, idx4] = ctx.get4(MASK, nib);
        let states = [self.stats[idx1], self.stats[idx2], self.stats[idx3], self.stats[idx4]];
        let new_states = NaiveStateTable::next4(states, nib);
        self.stats[idx1] = new_states[0];
        self.stats[idx2] = new_states[1];
        self.stats[idx3] = new_states[2];
        self.stats[idx4] = new_states[3];
    }
}
