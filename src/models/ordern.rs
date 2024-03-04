use super::{counter::Counter, Model};
use crate::usize;

pub struct OrderN {
    stats: Vec<Counter>,
    ctx: u32,
    history: u32,
    alignment: u8,
    bits_in_context: u8,
    alignment_bits: u8,
}

impl OrderN {
    pub fn new(bits_in_context: u8, alignment_bits: u8) -> Self {
        Self {
            stats: vec![Counter::new(); 1 << bits_in_context],
            ctx: 0,
            history: 0,
            alignment: 0,
            bits_in_context,
            alignment_bits,
        }
    }
}

impl Model for OrderN {
    fn predict(&self) -> u16 {
        self.stats[usize!(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[usize!(self.ctx)].update(bit);

        let mask_bits = self.bits_in_context - self.alignment_bits;
        let mask = (1 << mask_bits) - 1;
        let alignment_mask = (1 << self.alignment_bits) - 1;

        self.history = ((self.history << 1) | u32::from(bit)) & mask;
        self.alignment = (self.alignment + 1) & alignment_mask;
        self.ctx = (self.history << self.alignment_bits) | u32::from(self.alignment)
    }
}
