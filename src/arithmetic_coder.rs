use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;

use arithmetic_coder_io::*;

const PRECISION:  u32 = u32::BITS;            // 32
const PREC_SHIFT: u32 = PRECISION - 1;        // 31
const Q1:   u32 = 1 << (PRECISION - 2);       // 0x40000000, 1 = 0b01
const RMID: u32 = 2 << (PRECISION - 2);       // 0x80000000, 2 = 0b10
const Q3:   u32 = 3 << (PRECISION - 2);       // 0xC0000000, 3 = 0b11
const RLOW_MOD:  u32 = (1 << PREC_SHIFT) - 1; // 0x7FFFFFFF, AND with, set high bit 0, keep low bit (to 0 after shift)
const RHIGH_MOD: u32 = (1 << PREC_SHIFT) + 1; // 0x80000001, OR with, set high bit 1, set low bit 1

pub struct ArithmeticCoder<TWrite: Write, TRead: Read> {
    x1: u32, // low
    x2: u32, // high
    x: u32,  // state
    io: ArithmeticCoderIO<TWrite, TRead>
}

impl<TWrite, TRead> ArithmeticCoder<TWrite, TRead>
where TWrite: Write, TRead: Read {
    pub fn init_enc(stream: TWrite) -> Self {
        Self {
            x1: 0, x2: u32::MAX, x: 0,
            io: Encode(ACWriter::new(stream))
        }
    }

    pub fn init_dec(stream: TRead) -> Self {
        let mut x: u32 = 0;
        let mut r = ACReader::new(stream);

        for _ in 0..u32::BITS {
            x = (x << 1) | r.read_bit();
        }

        Self {
            x1: 0, x2: u32::MAX, x,
            io: Decode(r)
        }
    }

    pub fn encode4(&mut self, nib: u8, probs: [u16; 4]) {
        // TODO: Optimize to a single update of ranges?
        // TODO: is this more clear than a for loop with (nib >> (3-i)) & 1 and probs[i]?
        self.encode(nib >> 3, probs[0]);
        self.encode((nib >> 2) & 1, probs[1]);
        self.encode((nib >> 1) & 1, probs[2]);
        self.encode(nib & 1, probs[3]);
    }

    pub fn encode(&mut self, bit: u8, prob: u16) {
        let xmid = self.get_mid(prob);
        let w = self.io.as_enc();

        match bit {
            1 => self.x2 = xmid,
            _ => self.x1 = xmid + 1
        }

        while ((self.x1 ^ self.x2) >> PREC_SHIFT) == 0 {
            w.write_bit(self.x2 >> PREC_SHIFT);
            self.x1 <<= 1;
            self.x2 = (self.x2 << 1) | 1;
        }

        while self.x1 >= Q1 && self.x2 < Q3 {
            w.inc_parity();
            self.x1 = (self.x1 << 1) & RLOW_MOD;
            self.x2 = (self.x2 << 1) | RHIGH_MOD;
        }
    }

    pub fn decode(&mut self, prob: u16) -> u8 {
        let xmid = self.get_mid(prob);
        let bit = (self.x <= xmid).into();
        let r = self.io.as_dec();

        match bit {
            1 => self.x2 = xmid,
            _ => self.x1 = xmid + 1
        }

        while ((self.x1 ^ self.x2) >> PREC_SHIFT) == 0 {
            self.x = (self.x << 1) | r.read_bit();
            self.x1 <<= 1;
            self.x2 = (self.x2 << 1) | 1;
        }

        while self.x1 >= Q1 && self.x2 < Q3 {
            self.x1 = (self.x1 << 1) & RLOW_MOD;
            self.x2 = (self.x2 << 1) | RHIGH_MOD;
            self.x = ((self.x << 1) ^ RMID) | r.read_bit();
        }

        bit
    }

    fn get_mid(&self, prob: u16) -> u32 {
        let range = u64::from(self.x2 - self.x1);
        let prob = renorm_prob(prob);
        let lerped_range = (range * prob) >> (u64::BITS - u32::BITS);
        let xmid = self.x1 + lerped_range as u32;
        debug_assert!(xmid >= self.x1 && xmid < self.x2);
        xmid
    }

    pub fn flush(&mut self) {
        let w = self.io.as_enc();
        debug_assert!(self.x1 >> PREC_SHIFT == 0 && self.x2 >> PREC_SHIFT == 1);

        w.write_bit(1);
        w.flush(self.x1 >> (u32::BITS - u8::BITS));
    }
}

fn renorm_prob(prob: u16) -> u64 {
    let mut prob = u64::from(prob) << (u32::BITS - u16::BITS);
    if prob == 0 {
        prob = 1;
    }

    debug_assert!(prob > 0 && prob < u64::from(u32::MAX));
    prob
}

mod arithmetic_coder_io {
    use std::{io::{Write, Read}, convert::TryInto};
    use crate::bit_helpers::{BitWriter, BitReader};
    pub use ArithmeticCoderIO::{Encode, Decode};

    pub enum ArithmeticCoderIO<TWrite: Write, TRead: Read> {
        Encode(ACWriter<TWrite>),
        Decode(ACReader<TRead>)
    }

    impl<TWrite, TRead> ArithmeticCoderIO<TWrite, TRead>
    where TWrite: Write, TRead: Read {
        pub fn as_dec(&mut self) -> &mut ACReader<TRead> {
            match self {
                Decode(r) => r,
                Encode(_) => unsafe { debug_unreachable!("[AC] Tried to use reader in encode mode"); }
            }
        }
        
        pub fn as_enc(&mut self) -> &mut ACWriter<TWrite> {
            match self {
                Encode(w) => w,
                Decode(_) => unsafe { debug_unreachable!("[AC] Tried to use writer in decode mode") },
            }
        }
    }

    pub struct ACReader<TRead: Read> {
        reader: BitReader<TRead>
    }

    impl<TRead: Read> ACReader<TRead> {
        pub fn new(stream: TRead) -> Self {
            Self { reader: BitReader::new(stream) }
        }

        pub fn read_bit(&mut self) -> u32 {
            self.reader.read_bit().unwrap_or(0).into()
        }
    }

    pub struct ACWriter<TWrite: Write> {
        writer: BitWriter<TWrite>,
        rev_bits: u64
    }

    impl<TWrite: Write> ACWriter<TWrite> {
        pub fn new(stream: TWrite) -> Self {
            Self { writer: BitWriter::new(stream), rev_bits: 0 }
        }

        pub fn write_bit(&mut self, bit: u32) {
            let bit = bit.try_into().unwrap();
            self.writer.write_bit(bit);
    
            while self.rev_bits > 0 {
                self.writer.write_bit(bit ^ 1);
                self.rev_bits -= 1;
            }
        }

        pub fn inc_parity(&mut self) {
            self.rev_bits += 1;
        }

        pub fn flush(&mut self, pad_byte: u32) {
            let pad_byte = pad_byte.try_into().unwrap();
            self.writer.flush(pad_byte);
        }
    }
}
