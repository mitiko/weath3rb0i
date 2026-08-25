use super::History;
use crate::{
    entropy_coding::arithmetic_coder::{ACWrite, ArithmeticCoder},
    helpers::RotatingBuffer,
    models::Model,
    u8, Analytics,
};

pub struct ACHistory<M: Model> {
    pos: usize,
    bits: u64,
    probs: RotatingBuffer<u16, 64>,
    max_bits: u8,
    compressed_bits_count: usize,
    model: M,
}

impl<M: Model> ACHistory<M> {
    pub fn new(max_bits: u8, model: M) -> Self {
        Self {
            pos: 0,
            bits: 0,
            probs: RotatingBuffer::init(1 << 15),
            max_bits,
            compressed_bits_count: 0,
            model,
        }
    }
}

impl<M: Model> History for ACHistory<M> {
    fn update(&mut self, bit: u8) {
        let p = self.model.predict();
        self.model.update(bit);
        self.probs.push(p);

        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
        self.compressed_bits_count = 0;
    }

    fn hash(&mut self) -> u32 {
        // if self.pos == 100 {
        //     let mut entropy = 0.0;
        //     for i in 0..64 {
        //         let bit = u8!((self.bits >> i) & 1);
        //         let p = self.cache[i];
        //         let prob = f64::from(p) / 65536.0;
        //         let prob = if bit == 1 { prob } else { 1.0 - prob };
        //         entropy += -prob.log2();
        //         println!("bit={}, p={}, h={}", bit, p, entropy);
        //     }
        // }

        let mut ac = ArithmeticCoder::new_coder();
        let mut writer = EntropyWriter {
            state: 0,
            rev_bits: 0,
            idx: 0,
            max_bits: self.max_bits,
        };
        for i in 0..self.pos.min(64) {
            let bit = u8!((self.bits >> i) & 1);
            let p = self.probs[i];
            let res = ac.encode(bit, p, &mut writer);
            if res.is_err() {
                self.compressed_bits_count = i + 1;
                break;
            }
        }
        // _ = ac.flush(&mut writer);

        // writer.state >> (32 - writer.idx)
        writer.state.wrapping_shr(32 - writer.idx as u32)
    }
}

#[derive(Clone, Debug)]
struct EntropyWriter {
    state: u32,
    max_bits: u8,
    rev_bits: u16,
    idx: u8,
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
        self.write_bit(1)?;
        debug_assert!(self.rev_bits == 0);
        Ok(())
    }
}

impl<M: Model + Analytics> Analytics for ACHistory<M> {
    fn log(&mut self) -> serde_json::Value {
        serde_json::json!({
            "h": self.hash(),
            "bits": self.compressed_bits_count,
            "model": self.model.log(),
        })
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "history/ACHistory",
            "children": {
                "model": self.model.metadata(),
            },
            "vars": { "bits": "usize", },
            "description": "Compressed history with arithmetic coding using a static model",
        })
    }
}
