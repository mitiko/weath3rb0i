pub mod ac_history;
pub mod huff_history;

pub use self::{ac_history::*, huff_history::*};

pub trait History {
    fn update(&mut self, bit: u8);
    fn hash(&mut self) -> u32;
}

impl History for u32 {
    fn update(&mut self, bit: u8) {
        *self = (*self << 1) | u32::from(bit);
    }

    fn hash(&mut self) -> u32 {
        *self
    }
}

impl crate::Analytics for u32 {
    fn log(&mut self) -> serde_json::Value {
        serde_json::json!({ "h": *self })
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "history/Raw",
            "description": "The raw last 32 bits, unhashed",
        })
    }
}
