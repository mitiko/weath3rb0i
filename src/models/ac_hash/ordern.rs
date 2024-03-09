use crate::{models::{ACHashModel, Model, OrderN}, unroll_for, unroll_for_rev, usize};

#[derive(Clone)]
pub struct OrderNStationary {
    probs: Vec<u16>,
    ctx: u32,
    history: u32,
    alignment: u8,
    bits_in_context: u8,
    alignment_bits: u8,
}

impl OrderNStationary {
    pub fn new(buf: &[u8], bits_in_context: u8, alignment_bits: u8) -> Self {
        let mut model = OrderN::new(bits_in_context, alignment_bits);
        for byte in buf.iter().rev() {
            unroll_for_rev!(bit in byte, {
                model.update(bit);
            });
        }
        let probs = model.stats.iter().map(|c| c.p()).collect();
        Self {
            probs,
            ctx: 0,
            history: 0,
            alignment: 0,
            bits_in_context,
            alignment_bits,
        }
    }
}

impl ACHashModel for OrderNStationary {
    fn predict(&mut self, bit: u8) -> u16 {
        let p = self.probs[usize!(self.ctx)];

        let mask_bits = self.bits_in_context - self.alignment_bits;
        let mask = (1 << mask_bits) - 1;
        let alignment_mask = (1 << self.alignment_bits) - 1;

        self.history = ((self.history << 1) | u32::from(bit)) & mask;
        self.alignment = (self.alignment + 1) & alignment_mask;
        self.ctx = (self.history << self.alignment_bits) | u32::from(self.alignment);

        p
    }

    fn align(&mut self, alignment: u8) {
        self.history = 0;
        self.ctx = 0;
        self.alignment = (8 - alignment) & 7;
    }
}
