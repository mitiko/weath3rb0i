use super::History;
use crate::entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder};
use crate::models::StationaryModel;
use crate::models::{stationary::RevBitStationaryModel, Model};
use crate::u8;
use std::marker::PhantomData;

pub struct ACHistory {
    bits: u64, // TODO: u128?
    max_bits: u8,
    alignment: u8,
    is_big_endian: bool,
}

impl ACHistory {
    /// defaults to big endian
    pub fn new(max_bits: u8) -> Self {
        Self { bits: 0, alignment: 0, max_bits, is_big_endian: true }
    }

    pub fn new_with_endiannes(max_bits: u8, is_big_endian: bool) -> Self {
        Self { bits: 0, alignment: 0, max_bits, is_big_endian }
    }
}

impl History for ACHistory {
    fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u64::from(bit);
        self.alignment = (self.alignment + 1) & 7;
    }

    fn hash(&mut self) -> u32 {
        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = EntropyWriter { state: 0, rev_bits: 0, idx: 0, max_bits: self.max_bits, is_big_endian: self.is_big_endian };
        let mut model = RevBitStationaryModel::new(self.alignment);
        for i in 0..64 {
            let bit = u8!((self.bits >> i) & 1);
            let res = ac.encode(bit, model.predict(), &mut writer);
            if res.is_err() {
                break;
            }
        }

        if self.is_big_endian {
            writer.state >> (32 - writer.idx)
        } else {
            writer.state
        }
    }
}

// fn rev_bits(mut n: u32) -> u32 {
//     let mut x = 0;
//     for _ in 0..32 {
//         x <<= 1;
//         if n & 1 == 1 {
//             x = x | 1;
//         }
//         n >>= 1;
//     }
//     x
// }

#[derive(Clone, Debug)]
struct EntropyWriter {
    state: u32,
    max_bits: u8,
    rev_bits: u16,
    idx: u8,
    is_big_endian: bool,
}

impl ACWrite for EntropyWriter {
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        debug_assert!(self.idx <= self.max_bits);
        use std::io::{Error, ErrorKind};
        let bit = bit.try_into().unwrap_or_default();

        let mut write_bit_raw = |bit: u8| -> std::io::Result<()> {
            if self.idx == self.max_bits {
                return Err(Error::from(ErrorKind::Other));
            }
            self.state = if self.is_big_endian {
                (self.state >> 1) | (u32::from(bit) << 31)
            } else {
                (self.state << 1) | u32::from(bit)
            };
            self.idx += 1;
            Ok(())
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
