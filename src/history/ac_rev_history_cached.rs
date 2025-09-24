use crate::{
    entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder},
    history::{CappedEntropyWriter, History},
    models::{ACHashModel, Model},
    u8,
};
use std::collections::HashMap;

pub struct ACRevHistoryCached<M: ACHashModel> {
    pos: u64,
    bits: u64, // TODO: u128?
    max_bits: u8,
    model: M,
    cache_size: u8,
    cache: HashMap<(u64, u8, u8), (CappedEntropyWriter, ArithmeticCoder<CappedEntropyWriter>)>,
}

impl<M: ACHashModel> ACRevHistoryCached<M> {
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

impl<M: ACHashModel> History for ACRevHistoryCached<M> {
    fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
    }

    fn hash(&mut self) -> u32 {
        let alignment = u8!(self.pos & 7);

        let (c1, c2) = (self.cache_size, self.cache_size / 2);
        let (m1, m2) = ((1 << c1) - 1, (1 << c2) - 1);
        let (k1, k2) = (
            (self.bits & m1, alignment, 0),
            (self.bits & m2, alignment, 1),
        );
        let (start, mut writer, mut ac) = match self.cache.get(&k1) {
            Some((writer, ac)) => (c1, writer.clone(), ac.clone()),
            None => match self.cache.get(&k2) {
                Some((writer, ac)) => (c2, writer.clone(), ac.clone()),
                None => (
                    0,
                    CappedEntropyWriter::new(self.max_bits),
                    ArithmeticCoder::new_coder(),
                ),
            },
        };

        let alignment = ((alignment + 32) - start) & 7;
        self.model.align(alignment);
        for i in start..64 {
            let bit = u8!((self.bits >> i) & 1);
            let res = ac.encode(bit, self.model.predict(), &mut writer);
            if res.is_err() {
                break;
            }
            if i == c2 - 1 {
                self.cache.insert(k2, (writer.clone(), ac.clone()));
            }
            if i == c1 - 1 {
                self.cache.insert(k1, (writer.clone(), ac.clone()));
            }
        }

        writer.state >> (32 - writer.idx)
    }
}
