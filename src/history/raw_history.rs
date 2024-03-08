use super::History;

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
