use super::{counter::Counter, Model};

pub struct MX {
    stats: Vec<Counter>,
    ctx: u32,
    history: u32,
    alignment: u8,
    bits_in_context: u32,
    aligned: bool,
}

impl MX {
    pub fn new(bits_in_context: u32, aligned: bool) -> Self {
        Self {
            stats: vec![Counter::new(); 1 << bits_in_context],
            ctx: 0,
            history: 0,
            alignment: 0,
            bits_in_context,
            aligned,
        }
    }
}

impl Model for MX {
    fn predict(&self) -> u16 {
        self.stats[self.ctx as usize].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[self.ctx as usize].update(bit);
        // const MASK: u16 = (1 << (BITS - 3 + 1)) - 1;
        let mask_bits = if self.aligned {
            self.bits_in_context - 3
        } else {
            self.bits_in_context
        };
        let mask = (1 << mask_bits) - 1;
        self.alignment = (self.alignment + 1) & 7;
        self.history = ((self.history << 1) | u32::from(bit)) & mask;
        self.ctx = if self.aligned {
            (self.history << 3) | u32::from(self.alignment)
        } else {
            self.history
        };
    }
}

pub struct M16Unaligned {
    stats: Vec<Counter>,
    history: u16,
}

impl M16Unaligned {
    pub fn new() -> Self {
        Self { stats: vec![Counter::new(); 1 << 16], history: 0 }
    }
}

impl Model for M16Unaligned {
    fn predict(&self) -> u16 {
        self.stats[usize::from(self.history)].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[usize::from(self.history)].update(bit);
        self.history = (self.history << 1) | u16::from(bit);
    }
}


