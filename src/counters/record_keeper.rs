use super::RecordCounter;

#[derive(Clone, Copy)]
pub struct ExactRecordKeeper {
    prev: [u16; 2]
}

impl RecordCounter for ExactRecordKeeper {
    fn new() -> Self { Self { prev: [0; 2] } }

    fn predict(&self, pos: u16) -> u16 {
        match (self.prev[0] == pos, self.prev[1] == pos) {
            (true, false) => 0,
            (false, true) => u16::MAX,
            _ => 1 << 15, // half
        }
    }

    fn update(&mut self, pos: u16, bit: u8) {
        self.prev[usize::from(bit)] = pos;
    }
}
