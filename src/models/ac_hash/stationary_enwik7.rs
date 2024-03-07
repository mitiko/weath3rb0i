use crate::models::ACHashModel;

// big endian alignment
// generated from gen-stationary-eh-ac-model
const PROB_TABLE: [u16; 8] = [752, 50314, 58928, 21421, 24680, 30788, 24297, 32530];

// encodes bits in reverse
pub struct Enwik7StationaryModel {
    alignment: u8,
}

impl Enwik7StationaryModel {
    pub fn new() -> Self {
        Self { alignment: 0 }
    }
}

impl ACHashModel for Enwik7StationaryModel {
    fn align(&mut self, alignment: u8) {
        self.alignment = alignment;
    }

    fn predict(&mut self) -> u16 {
        self.alignment = (self.alignment + 7) & 7; // -1 = 7 (mod 8)
        PROB_TABLE[usize::from(self.alignment)]
    }
}
