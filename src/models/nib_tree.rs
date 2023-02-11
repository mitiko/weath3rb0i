pub struct NibTree {
    /// Bit id in nibble 0-3
    pub bit_id: u8,
    pub cache: u8
}

impl NibTree {
    pub fn new() -> Self {
        Self { bit_id: 0, cache: 0 }
    }

    pub fn get4(&self, nib: u8) -> [usize; 4] {[
        0,
        usize::from(1 + (nib >> 3)),
        usize::from(3 + (nib >> 2)),
        usize::from(7 + (nib >> 1))
    ]}

    pub fn get(&self) -> usize {
        usize::from((1u8 << self.bit_id) - 1 + self.cache)
    }

    pub fn update(&mut self, bit: u8) {
        self.cache = (self.cache << 1) | bit;
        self.bit_id = (self.bit_id + 1) & 3;
        if self.bit_id == 0 { self.cache = 0; }
    }
}
