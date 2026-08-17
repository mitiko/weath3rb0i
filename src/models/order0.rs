use super::{counter::Counter, AdaptiveModel};
use crate::Analytics;

#[derive(Clone)]
pub struct Order0 {
    stats: [Counter; 1 << 11],
    history: u8,
    alignment: u8,
    ctx: u16,
}

impl Order0 {
    pub fn new() -> Self {
        Self {
            stats: [Counter::new(); 1 << 11],
            history: 0,
            alignment: 0,
            ctx: 0,
        }
    }
}

impl AdaptiveModel for Order0 {
    fn predict(&self) -> u16 {
        self.stats[usize::from(self.ctx)].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[usize::from(self.ctx)].update(bit);
    }

    fn update(&mut self, bit: u8) {
        self.history = (self.history << 1) | bit;
        self.alignment = (self.alignment + 1) % 8;
        self.ctx = u16::from(self.alignment) << 8 | u16::from(self.history);
    }
}

impl Analytics for Order0 {
    fn log(&mut self) -> serde_json::Value {
        let history = self.history;
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
            "type": "model/Order0",
            "description": "Order-0 model with 8-bit history and 11-bit counter",
            "children": {
                "counter": Counter::metadata(),
            },
            "vars": { "align": "u8" },
            // model-specific
            "is_adaptive": true,
        })
    }
}
