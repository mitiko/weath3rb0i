use super::{counter::Counter, Model};
use crate::history::History;
use crate::usize;

pub struct OrderNEntropy<H: History> {
    stats: Vec<Counter>,
    ctx: u32,
    history: H,
    alignment: u8,
    bits_in_context: u8,
    alignment_bits: u8,
}

impl<H: History> OrderNEntropy<H> {
    pub fn new(bits_in_context: u8, alignment_bits: u8, history: H) -> Self {
        Self {
            stats: vec![Counter::new(); 1 << bits_in_context],
            ctx: 0,
            alignment: 0,
            history,
            bits_in_context,
            alignment_bits,
        }
    }
}

impl<H: History> Model for OrderNEntropy<H> {
    fn predict(&self) -> u16 {
        self.stats[usize!(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[usize!(self.ctx)].update(bit);

        let mask_bits = self.bits_in_context - self.alignment_bits;
        let mask = (1 << mask_bits) - 1;
        let alignment_mask = (1 << self.alignment_bits) - 1;

        self.history.update(bit);
        self.alignment = (self.alignment + 1) & alignment_mask;
        let hash = self.history.hash() & mask;
        self.ctx = (hash << self.alignment_bits) | u32::from(self.alignment);
    }
}
