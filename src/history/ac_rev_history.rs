use crate::{
    entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder},
    history::{CappedEntropyWriter, History},
    models::{ACHashModel, Model},
    u8,
};
use std::marker::PhantomData;

pub struct ACRevHistory<M: ACHashModel> {
    pos: u64,
    bits: u64, // TODO: u128?
    max_bits: u8,
    model: M,
}

impl<M: ACHashModel> ACRevHistory<M> {
    pub fn new(max_bits: u8, model: M) -> Self {
        Self { pos: 0, bits: 0, max_bits, model }
    }
}

impl<M: ACHashModel> History for ACRevHistory<M> {
    fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
    }

    fn hash(&mut self) -> u32 {
        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = CappedEntropyWriter {
            state: 0,
            rev_bits: 0,
            idx: 0,
            max_bits: self.max_bits,
        };
        self.model.align(u8!(self.pos & 7));
        for i in 0..64 {
            let bit = u8!((self.bits >> i) & 1);
            let res = ac.encode(bit, self.model.predict(), &mut writer);
            if res.is_err() {
                break;
            }
        }

        writer.state >> (32 - writer.idx)
    }
}
