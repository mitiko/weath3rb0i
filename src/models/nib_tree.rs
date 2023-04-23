pub struct NibTree {
    /// Bit id in nibble 0-3
    bit_id: u8,
    cache: u8,
}

impl NibTree {
    pub fn new() -> Self {
        Self { bit_id: 0, cache: 0 }
    }

    pub fn get4(&self, nib: u8) -> [usize; 4] {
        let nib = usize::from(nib);
        [0, 1 + (nib >> 3), 3 + (nib >> 2), 7 + (nib >> 1)]
    }

    pub fn get(&self) -> usize {
        usize::from((1 << self.bit_id) - 1 + self.cache)
    }

    pub fn update(&mut self, bit: u8) -> Option<u8> {
        self.cache = (self.cache << 1) | bit;
        self.bit_id = (self.bit_id + 1) & 3;
        (self.bit_id == 0).then(|| std::mem::take(&mut self.cache))
    }
}
