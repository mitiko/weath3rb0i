use crate::entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder};
use crate::models::{stationary::RevBitStationaryModel, StationaryModel};
use crate::u8;
use std::collections::HashMap;

pub mod raw_history;
pub mod ac_history;
pub use raw_history::*;
pub use ac_history::*;

pub trait History {
    fn update(&mut self, bit: u8);
    fn hash(&mut self) -> u32;
}

pub struct HistoryX {
    bits: u64,
    alignment: u8,
    cache: HashMap<(u8, u8), (EntropyWriter, ArithmeticCoder<EntropyWriter>)>,
}

impl HistoryX {
    pub fn new() -> Self {
        Self { bits: 0, alignment: 0, cache: HashMap::new() }
    }
}

impl History for HistoryX {
    fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u64::from(bit);
        self.alignment = (self.alignment + 1) % 8;
    }

    fn hash(&mut self) -> u32 {
        let last_byte = u8!(self.bits & 0xff);
        let cached_state = self.cache.get(&(last_byte, self.alignment));
        let (mut writer, mut ac) = match cached_state {
            Some((writer, ac)) => (writer.clone(), ac.clone()),
            None => (
                EntropyWriter { state: 0, rev_bits: 0, idx: 0 },
                ArithmeticCoder::new_coder(),
            ),
        };
        let mut model = RevBitStationaryModel::new(self.alignment);
        let mut i = if cached_state.is_some() { 8 } else { 0 };

        while i < u64::BITS {
            let bit = u8!((self.bits >> i) & 1);
            let res = ac.encode(bit, model.predict(), &mut writer);
            i += 1;
            if i == 8 {
                self.cache
                    .insert((last_byte, self.alignment), (writer.clone(), ac.clone()));
            }
            if res.is_err() {
                break;
            }
        }

        if i < 8 {
            self.cache
                .insert((last_byte, self.alignment), (writer.clone(), ac.clone()));
        }

        (u32::from(writer.state) << 3) | u32::from(self.alignment)
    }
}

#[derive(Clone, Debug)]
struct EntropyWriter {
    state: u8,
    rev_bits: u16,
    idx: u8,
}

impl ACWrite for EntropyWriter {
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> std::io::Result<()> {
        debug_assert!(self.idx <= 8);
        use std::io::{Error, ErrorKind};
        let bit: u8 = bit.try_into().unwrap_or_default();

        let mut write_bit_raw = |bit: u8| -> std::io::Result<()> {
            if self.idx == 8 {
                return Err(Error::from(ErrorKind::Other));
            }
            self.state = (self.state << 1) | bit;
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
