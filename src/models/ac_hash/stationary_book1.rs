use crate::models::ACHashModel;

// big endian alignment
// generated from gen-stationary-eh-ac-model
const PROB_TABLE: [u16; 8] = [1, 50188, 62497, 15819, 22545, 31499, 22988, 29616];

// encodes bits in reverse
pub struct Book1StationaryModel {
    alignment: u8,
}

impl Book1StationaryModel {
    pub fn new() -> Self {
        Self { alignment: 0 }
    }
}

impl ACHashModel for Book1StationaryModel {
    fn align(&mut self, alignment: u8) {
        self.alignment = alignment;
    }

    fn predict(&mut self) -> u16 {
        self.alignment = (self.alignment + 7) & 7; // -1 = 7 (mod 8)
        PROB_TABLE[usize::from(self.alignment)]
    }
}
