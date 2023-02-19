use super::{Counter, Model};

#[derive(Clone, Copy)]
pub struct BitCounter {
    data: [u16; 2]
}

impl Counter for BitCounter {}

impl Model for BitCounter {
    fn new() -> Self { Self { data: [0; 2] } }

    fn predict(&self) -> u16 {
        let c0 = u64::from(self.data[0]);
        let c1 = u64::from(self.data[1]);
        let p = (1 << 17) * (c1 + 1) / (c0 + c1 + 2);
        u16::try_from((p >> 1) + (p & 1)).unwrap() // rounding
    }

    fn update(&mut self, bit: u8) {
        self.data[usize::from(bit)] += 1;
        if self.data[usize::from(bit)] == u16::MAX {
            self.data[0] = (self.data[0] >> 1) + (self.data[0] & 1);
            self.data[1] = (self.data[1] >> 1) + (self.data[1] & 1);
        }
    }
}
