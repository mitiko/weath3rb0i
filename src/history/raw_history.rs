use super::History;
use crate::Analytics;

pub struct RawHistory {
    bits: u32,
}

impl RawHistory {
    pub fn new() -> Self {
        Self { bits: 0 }
    }
}

impl History for RawHistory {
    fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u32::from(bit);
    }

    fn hash(&mut self) -> u32 {
        self.bits
    }
}

impl Analytics for RawHistory {
    fn log(&mut self) -> serde_json::Value {
        serde_json::json!({
            "h": self.hash()
        })
    }

    fn metadata() -> serde_json::Value {
        serde_json::json!({
            "type": "history/RawHistory",
            "description": "Stores the raw bits of the history in u32",
        })
    }
}
