use super::RecordCounter;

#[derive(Clone, Copy)]
pub struct ExactRecordKeeper {
    prev: [u16; 2],
    record: [u16; 2]
}

impl RecordCounter for ExactRecordKeeper {
    fn new() -> Self { Self { prev: [0; 2], record: [0; 2] } }

    fn predict(&self, pos: u16) -> u16 {
        let p0 = self.prev[0].wrapping_add(self.record[0]);
        let p1 = self.prev[1].wrapping_add(self.record[1]);
        match (pos == p0, pos == p1) {
            (true, false) => 0,
            (false, true) => u16::MAX,
            _ => 1 << 15, // half
        }
    }

    fn update(&mut self, pos: u16, bit: u8) {
        let bit = usize::from(bit);
        self.record[bit] = pos.wrapping_sub(self.prev[bit]);
        self.prev[bit] = pos;
    }
}
