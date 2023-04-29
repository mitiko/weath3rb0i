use super::StationaryModel;
use super::{counter::Counter, Model};
use super::stationary::{PROB_TABLE, RevBitStationaryModel};

pub struct Order0Entropy {
    stats: [Counter; 1 << 11],
    history: History, // TODO: share history across big models
    alignment: u8,
    ctx: u16,
}

impl Order0Entropy {
    pub fn new() -> Self {
        Self {
            stats: [Counter::new(); 1 << 11],
            ctx: 0,
            alignment: 0,
            history: History::new(),
        }
    }
}

impl Model for Order0Entropy {
    fn predict(&self) -> u16 {
        self.stats[usize::from(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[usize::from(self.ctx)].update(bit);
        self.alignment = (self.alignment + 1) % 8;
        self.history.update(bit);
        self.ctx = u16::from(self.history.hash()) << 3 | u16::from(self.alignment);
    }
}

struct History {
    bits: u64,
    alignment: u8
}

impl History {
    fn new() -> Self {
        Self { bits: 0, alignment: 0 }
    }

    fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u64::from(bit);
        self.alignment = (self.alignment + 1) % 8;
    }

    fn hash(&self) -> u8 {
        let mut model = RevBitStationaryModel::new(self.alignment);
        let mut writer = EntropyWriter { state: 0, rev_bits: 0, idx: 0 };
        let mut ac = ArithmeticCoder::new_coder();

        for i in 0..u64::BITS {
            let bit = u8::try_from((self.bits >> i) & 1).unwrap();
            let res = ac.encode(bit, model.predict(), &mut writer);
            if res.is_err() {
                break;
            }
        }
        writer.state
    }
}

struct EntropyWriter {
    state: u8,
    rev_bits: u16,
    idx: u8,
}

use crate::entropy_coding::{ACWrite, ArithmeticCoder};
impl ACWrite for EntropyWriter {
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        use std::io::{ErrorKind, Error};
        let bit: u8 = bit.try_into().unwrap_or_default();
        self.state = (self.state << 1) | bit;

        self.idx += 1;
        if self.idx == 8 {
            return Err(Error::from(ErrorKind::OutOfMemory));
        }

        while self.rev_bits > 0 {
            self.rev_bits -= 1;
            self.state = (self.state << 1) | (bit ^ 1);
            self.idx += 1;
            if self.idx == 8 {
                return Err(Error::from(ErrorKind::OutOfMemory));
            }
        }
        Ok(())
    }

    fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    fn flush(&mut self, _padding: u32) -> std::io::Result<()> {
        unimplemented!()
    }
}
