use super::{counter::Counter, Model};

pub struct Order0Entropy {
    stats: [Counter; 1 << 11],
    hash: u8,
    alignment: u8,
    context: Context,
}

impl Order0Entropy {
    pub fn new() -> Self {
        Self {
            stats: [Counter::new(); 1 << 11],
            hash: 0,
            alignment: 0,
            context: Context::new(),
        }
    }
}

impl Model for Order0Entropy {
    fn predict(&self) -> u16 {
        let ctx = usize::from(self.hash) << 3 | usize::from(self.alignment);
        self.stats[ctx].p()
    }

    fn update(&mut self, bit: u8) {
        let ctx = usize::from(self.hash) << 3 | usize::from(self.alignment);
        self.stats[ctx].update(bit);
        self.alignment = (self.alignment + 1) % 8;
        self.context.update(bit);
        self.hash = self.context.hash();
    }
}

#[derive(Copy, Clone)]
pub struct ConstCounter {
    data: [u16; 2],
}

impl ConstCounter {
    pub const fn new() -> Self {
        Self { data: [0; 2] }
    }

    pub const fn p(&self) -> u16 {
        let c0 = self.data[0] as u64;
        let c1 = self.data[1] as u64;
        let p = (1 << 17) * (c1 + 1) / (c0 + c1 + 2);
        ((p >> 1) + (p & 1)) as u16 // rounding
    }

    pub const fn update(mut self, bit: u8) -> Self {
        self.data[bit as usize] += 1;
        if self.data[bit as usize] == u16::MAX {
            self.data[0] = (self.data[0] >> 1) + (self.data[0] & 1);
            self.data[1] = (self.data[1] >> 1) + (self.data[1] & 1);
        }
        self
    }
}

const PROB_TABLE: [u16; 8] = {
    let string = include_str!("/data/book1");
    let mut model = [ConstCounter::new(); 8];
    let data = string.as_bytes();
    let mut idx = 0;
    while idx < 100_000 {
        let byte = data[idx];
        // Little Endian
        model[0] = model[0].update(byte & 1);
        model[1] = model[1].update((byte >> 1) & 1);
        model[2] = model[2].update((byte >> 2) & 1);
        model[3] = model[3].update((byte >> 3) & 1);
        model[4] = model[4].update((byte >> 4) & 1);
        model[5] = model[5].update((byte >> 5) & 1);
        model[6] = model[6].update((byte >> 6) & 1);
        model[7] = model[7].update(byte >> 7);
        idx += 1;
    }
    [
        model[0].p(),
        model[1].p(),
        model[2].p(),
        model[3].p(),
        model[4].p(),
        model[5].p(),
        model[6].p(),
        model[7].p(),
    ]
};

struct Context {
    history: u32,
    seen: std::collections::HashMap<u8, u32>
}

impl Context {
    fn new() -> Self {
        let mut model = [Counter::new(); 8];
        let mut freq: [u64; 256] = [0; 256];
        let data = std::fs::read("/data/book1").unwrap();
        for byte in data {
            freq[usize::from(byte)] += 1;
            model
                .iter_mut()
                .enumerate()
                .for_each(|(i, c)| c.update((byte >> i) & 1));
        }
        let table: Vec<_> = model.iter().map(|c| c.p()).collect();
        dbg!(table); // little endian

        Self { history: 0, seen: std::collections::HashMap::new() }
    }
    fn update(&mut self, bit: u8) {
        self.history = (self.history << 1) | u32::from(bit);
    }

    fn hash(&mut self) -> u8 {
        let mut writer = EntropyWriter::new();
        let mut ac = ArithmeticCoder::new_coder();
        let mut bits = self.history;
        for i in 0..32 {
            let bit = u8::try_from(bits & 1).unwrap();
            let prob = PROB_TABLE[i % 8];

            let res = ac.encode(bit, prob, &mut writer);
            if res.is_err() {
                break;
            }
            bits >>= 1;
        }
        let hash = writer.state;
        if self.seen.contains_key(&hash) {
            println!("{hash} -> {}", self.history >> 24);
        }
        self.seen.insert(hash, self.history);
        hash
    }
}

struct EntropyWriter {
    state: u8,
    rev_bits: u16,
    idx: u8,
}

impl EntropyWriter {
    fn new() -> Self {
        Self { state: 0, rev_bits: 0, idx: 0 }
    }
}

use crate::entropy_coding::{ACWrite, ArithmeticCoder};
impl ACWrite for EntropyWriter {
    fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        let bit: u8 = bit.try_into().unwrap_or_default();

        self.state = (self.state << 1) | bit;
        self.idx += 1;
        if self.idx == 8 {
            return Err(std::io::Error::from(std::io::ErrorKind::OutOfMemory));
        }

        while self.rev_bits > 0 {
            self.rev_bits -= 1;
            self.state = (self.state << 1) | (bit ^ 1);
            self.idx += 1;
            if self.idx == 8 {
                return Err(std::io::Error::from(std::io::ErrorKind::OutOfMemory));
            }
        }
        Ok(())
    }

    fn flush(&mut self, _padding: u32) -> std::io::Result<()> {
        unimplemented!()
    }
}
