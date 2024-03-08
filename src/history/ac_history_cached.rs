use super::History;
use crate::u8;
use crate::{
    entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder},
    models::{ACHashModel, Model},
};
use std::collections::HashMap;

pub struct ACHistoryCached<M: ACHashModel> {
    pos: u64,
    bits: u64, // TODO: u128?
    max_bits: u8,
    model: M,
    cache_size: u8,
    cache: HashMap<(u64, u8), (EntropyWriter, ArithmeticCoder<EntropyWriter>)>,
}

impl<M: ACHashModel> ACHistoryCached<M> {
    pub fn new(max_bits: u8, model: M, cache_size: u8) -> Self {
        Self {
            pos: 0,
            bits: 0,
            max_bits,
            model,
            cache_size,
            cache: HashMap::new(),
        }
    }
}

impl<M: ACHashModel> History for ACHistoryCached<M> {
    fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
    }

    fn hash(&mut self) -> u32 {
        let alignment = u8!(self.pos & 7);
        let mask = (1 << self.cache_size) - 1;
        let last_bits = self.bits & mask;
        let key = (last_bits, alignment);
        let cache_entry = self.cache.get(&key);
        let (mut writer, mut ac) = match cache_entry {
            Some((writer, ac)) => (writer.clone(), ac.clone()),
            None => (
                EntropyWriter::new(self.max_bits),
                ArithmeticCoder::new_coder(),
            ),
        };

        let start = if cache_entry.is_some() {
            let alignment = ((alignment + 32) - self.cache_size) & 7;
            self.model.align(alignment);
            self.cache_size
        } else {
            self.model.align(alignment);
            0
        };
        for i in start..64 {
            let bit = u8!((self.bits >> i) & 1);
            let res = ac.encode(bit, self.model.predict(), &mut writer);
            if res.is_err() {
                break;
            }
            if i == self.cache_size - 1 {
                self.cache.insert(key, (writer.clone(), ac.clone()));
            }
        }

        writer.state >> (32 - writer.idx)
    }
}

#[derive(Clone, Debug)]
struct EntropyWriter {
    state: u32,
    max_bits: u8,
    rev_bits: u16,
    idx: u8,
}

impl EntropyWriter {
    fn new(max_bits: u8) -> Self {
        Self { state: 0, max_bits, rev_bits: 0, idx: 0 }
    }
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
