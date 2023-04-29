use crate::entropy_coding::{ACWrite, ArithmeticCoder};
use crate::models::{stationary::RevBitStationaryModel, StationaryModel};

pub struct History {
    bits: u64,
    alignment: u8,
}

impl History {
    pub fn new() -> Self {
        Self { bits: 0, alignment: 0 }
    }

    pub fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u64::from(bit);
        self.alignment = (self.alignment + 1) % 8;
    }

    pub fn hash(&self) -> u16 {
        let mut model = RevBitStationaryModel::new(self.alignment);
        let mut writer = EntropyWriter { state: 0, rev_bits: 0, idx: 0 };
        let mut ac = ArithmeticCoder::new_coder();

        (0..u64::BITS)
            .map(|i| u8::try_from((self.bits >> i) & 1).unwrap())
            .map(|bit| ac.encode(bit, model.predict(), &mut writer))
            .take_while(|res| res.is_ok())
            .for_each(|_| {});

        (u16::from(writer.state) << 3) | u16::from(self.alignment)
    }
}

struct EntropyWriter {
    state: u8,
    rev_bits: u16,
    idx: u8,
}

impl ACWrite for EntropyWriter {
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        use std::io::{Error, ErrorKind};
        let bit: u8 = bit.try_into().unwrap_or_default();

        let mut write_bit_raw = |bit: u8| -> std::io::Result<()> {
            self.state = (self.state << 1) | bit;
            self.idx += 1;
            if self.idx == 8 {
                Err(Error::from(ErrorKind::Other))
            } else {
                Ok(())
            }
        };

        write_bit_raw(bit)?;
        while self.rev_bits > 0 {
            self.rev_bits -= 1;
            write_bit_raw(bit ^ 1)?;
        }
        Ok(())
    }

    fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    fn flush(&mut self, _padding: u32) -> std::io::Result<()> {
        unimplemented!("Entropy writer doesn't implement flushing")
    }
}
