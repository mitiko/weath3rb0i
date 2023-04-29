use super::StationaryModel;

// uses stats from book1
// TODO: Read from book1 for real
// big endian alignment
pub const PROB_TABLE: [u16; 8] = [1, 50188, 62497, 15819, 22545, 31499, 22988, 29616];

// encodes bits in reverse
pub struct RevBitStationaryModel {
    alignment: u8,
}

impl RevBitStationaryModel {
    pub fn new(alignment: u8) -> Self {
        Self { alignment }
    }
}

impl StationaryModel for RevBitStationaryModel {
    fn predict(&mut self) -> u16 {
        self.alignment = (self.alignment + 7) % 8; // -1 = 7 (mod 8)
        PROB_TABLE[usize::from(self.alignment)]
    }
}
