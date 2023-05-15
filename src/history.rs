// use crate::entropy_coding::{ACWrite, ArithmeticCoder};
// use crate::models::{stationary::RevBitStationaryModel, StationaryModel};

pub struct History {
    alignment: u16,
    state: u32,
}

const PROB_TABLE: [u16; 8] = [1, 50188, 62497, 15819, 22545, 31499, 22988, 29616];
const ONE: u32 = 1 << 16;

impl History {
    pub fn new() -> Self {
        Self { alignment: 0, state: 0 }
    }

    pub fn update(&mut self, bit: u8) {
        let p: u16 = PROB_TABLE[usize::from(self.alignment)];
        let p = u32::from(p);
        self.state = if bit == 1 {
            self.state * p
        } else {
            self.state * (ONE - p) + p
        };
        self.state >>= 15;
        self.state = (self.state >> 1) + (self.state & 1);
        self.alignment = (self.alignment + 1) % 8;
    }

    pub fn hash(&mut self) -> u16 {
        // -> 512'968
        (self.alignment << 8) | u16::try_from(self.state & 0xff).unwrap()
        // -> 547'139
        // self.alignment
    }
}
