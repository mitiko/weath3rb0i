pub mod ac_history;
pub mod ac_rev_history;
pub mod ac_rev_history_cached;
pub mod huff_history;
pub mod raw_history;

pub use self::{
    ac_history::*, ac_rev_history::*, ac_rev_history_cached::*, huff_history::*, raw_history::*,
};

pub trait History {
    fn update(&mut self, bit: u8);
    fn hash(&mut self) -> u32;
}

#[derive(Clone, Debug)]
struct CappedEntropyWriter {
    state: u32,
    max_bits: u8,
    rev_bits: u16,
    idx: u8,
}

impl CappedEntropyWriter {
    fn new(max_bits: u8) -> Self {
        Self { state: 0, max_bits, rev_bits: 0, idx: 0 }
    }
}

impl crate::entropy_coding::arithmetic_coder::ACWrite for CappedEntropyWriter {
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        debug_assert!(self.idx <= self.max_bits);
        use std::io::{Error, ErrorKind};
        let bit = bit.try_into().unwrap_or_default();

        let mut write_bit_raw = |bit: u8| -> std::io::Result<()> {
            if self.idx == self.max_bits {
                return Err(Error::from(ErrorKind::Other));
            }

            self.state = (self.state >> 1) | (u32::from(bit) << 31);
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

#[derive(Clone, Debug)]
struct EntropyWriter {
    state: u32,
    rev_bits: u16,
}

impl EntropyWriter {
    fn new() -> Self {
        Self { state: 0, rev_bits: 0 }
    }
}

impl crate::entropy_coding::arithmetic_coder::ACWrite for EntropyWriter {
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        use std::io::{Error, ErrorKind};
        let bit = bit.try_into().unwrap_or_default();

        // self.state = (self.state >> 1) | (u32::from(bit) << 31);
        self.state = (self.state << 1) | u32::from(bit);
        while self.rev_bits > 0 {
            self.rev_bits -= 1;
            // self.state = (self.state >> 1) | (u32::from(bit ^ 1) << 31);
            self.state = (self.state << 1) | u32::from(bit ^ 1);
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
