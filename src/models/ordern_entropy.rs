use super::{counter::Counter, AdaptiveModel};
use crate::history::History;
use crate::models::Order0;
use crate::{u16, usize, Analytics};

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

impl<H: History> AdaptiveModel for OrderNEntropy<H> {
    fn predict(&self) -> u16 {
        self.stats[usize!(self.ctx)].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[usize!(self.ctx)].update(bit);
    }

    fn update(&mut self, bit: u8) {
        let mask_bits = self.bits_in_context - self.alignment_bits;
        let mask = (1 << mask_bits) - 1;
        let alignment_mask = (1 << self.alignment_bits) - 1;

        self.history.update(bit);
        self.alignment = (self.alignment + 1) & alignment_mask;
        let hash = self.history.hash() & mask;
        self.ctx = (hash << self.alignment_bits) | u32::from(self.alignment);
    }
}

struct Order0Generic<H: History> {
    stats: [Counter; 1 << 11],
    history: H,
    alignment: u16,
    ctx: u16,
}

impl<H: History> Order0Generic<H> {
    pub fn new(history: H) -> Self {
        Self {
            stats: [Counter::new(); 1 << 11],
            history,
            alignment: 0,
            ctx: 0,
        }
    }
}

impl<H: History> AdaptiveModel for Order0Generic<H> {
    fn predict(&self) -> u16 {
        self.stats[usize::from(self.ctx)].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[usize::from(self.ctx)].update(bit);
    }

    fn update(&mut self, bit: u8) {
        self.history.update(bit);
        let hash = u16!(self.history.hash() & 0xff);

        self.alignment = (self.alignment + 1) % 8;
        self.ctx = (self.alignment << 8) | hash;
    }
}

impl<H: History + Analytics> Analytics for Order0Generic<H> {
    fn log(&mut self) -> serde_json::Value {
        let history = self.history.log();
        let stats = self.stats[usize::from(self.ctx)].log();
        let p = self.stats[usize::from(self.ctx)].p();

        serde_json::json!({
            "history": history,
            "counter": stats,
            "align": self.alignment,
            // model specific: probability & state
            "p": p,
            "s": self.ctx,
        })
    }

    fn metadata() -> serde_json::Value {
        serde_json::json!({
            "type": "model/Order0Generic",
            "description": "Order-0 model with generic history and counter",
            "children": {
                "history": H::metadata(),
                "counter": Counter::metadata(),
            },
            "vars": { "align": "u16" },
        })
    }
}
