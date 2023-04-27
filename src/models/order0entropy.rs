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
        // log hash + alignment
        // print!("h:{} @ {} <- {}", self.hash, self.alignment, self.context.history);
        // if self.alignment == 0 {
        //     let hb = self.context.history.to_be_bytes();
        //     let mut ext = String::new();
        //     for bb in hb {
        //         if bb.is_ascii_alphanumeric() {
        //             let chr = char::from_u32(bb as u32).unwrap();
        //             ext += format!(" {}", chr).as_str();
        //         } else {
        //             ext += format!(" 0x{}{}", bb >> 4, bb & 0xf).as_str();
        //         }
        //     }
        //     println!(" = {ext}");
        // }
        // else {
        //     println!();
        // }
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

const PROB_TABLE: [u16; 8] = [29616, 22988, 31499, 22545, 15819, 62497, 50188, 1];

struct Context {
    history: u64,
    idx: usize
}

impl Context {
    fn new() -> Self {
        Self { history: 0, idx: 0 }
    }

    fn update(&mut self, bit: u8) {
        self.history = (self.history << 1) | u64::from(bit);
        self.idx = (self.idx + 1) % 8;
    }

    fn hash(&mut self) -> u8 {
        let mut writer = EntropyWriter::new();
        let mut ac = ArithmeticCoder::new_coder();
        let mut bits = self.history;
        for i in 0..64 {
            let bit = u8::try_from(bits & 1).unwrap();
            let prob = PROB_TABLE[(8 + i - self.idx) % 8];

            let res = ac.encode(bit, prob, &mut writer);
            if res.is_err() {
                if i > 40 {
                    println!("compressed {i} bits into 8, {}, {}", writer.state, self.history);
                }
                break;
            }
            bits >>= 1;
        }
        writer.state
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
