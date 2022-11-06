// #![warn(missing_docs)]

use std::io::{Read, Write, self};

use super::{ACEncoder, ACDecoder};
use super::ac_io::{ACReader, ACWriter};

const PRECISION:  u32 = u32::BITS;            // 32
const PREC_SHIFT: u32 = PRECISION - 1;        // 31
const Q1:   u32 = 1 << (PRECISION - 2);       // 0x40000000, 1 = 0b01, quarter 1
const RMID: u32 = 2 << (PRECISION - 2);       // 0x80000000, 2 = 0b10, range mid
const Q3:   u32 = 3 << (PRECISION - 2);       // 0xC0000000, 3 = 0b11, quarter 3
const RLOW_MOD:  u32 = (1 << PREC_SHIFT) - 1; // 0x7FFFFFFF, range low modifier, AND with -> sets high bit to 0, keeps low bit (to 0 after shift)
const RHIGH_MOD: u32 = (1 << PREC_SHIFT) + 1; // 0x80000001, range high modifier, OR with -> sets high bit to 1, sets low bit to 1

pub struct ArithmeticCoder<W> {
    x1: u32, // low
    x2: u32, // high
    io: ACWriter<W>
}

impl<W: Write> ArithmeticCoder<W> {
    pub fn new(writer: W) -> Self {
        Self { io: ACWriter::new(writer), x1: 0, x2: u32::MAX }
    }

    pub fn encode4(&mut self, nib: u8, probs: [u16; 4]) -> io::Result<()> {
        self.encode(nib >> 3, probs[0])?;
        self.encode((nib >> 2) & 1, probs[1])?;
        self.encode((nib >> 1) & 1, probs[2])?;
        self.encode(nib & 1, probs[3])
    }
}

impl<W: Write> ACEncoder for ArithmeticCoder<W> {
    #[inline(never)]
    fn encode(&mut self, bit: u8, prob: u16) -> io::Result<()> {
        let xmid = lerp(self.x1, self.x2, prob);

        // Update range (kinda like binary search)
        match bit {
            1 => self.x2 = xmid,
            _ => self.x1 = xmid + 1
        }

        // Renormalize range -> write matching bits to stream
        while ((self.x1 ^ self.x2) >> PREC_SHIFT) == 0 {
            self.io.write_bit(self.x2 >> PREC_SHIFT)?;
            self.x1 <<= 1;
            self.x2 = (self.x2 << 1) | 1;
        }

        // E3 renorm (special case) -> increase parity bits but don't write anything to stream
        while self.x1 >= Q1 && self.x2 < Q3 {
            self.io.inc_parity();
            self.x1 = (self.x1 << 1) & RLOW_MOD;
            self.x2 = (self.x2 << 1) | RHIGH_MOD;
        }

        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        debug_assert!(self.x1 >> PREC_SHIFT == 0 && self.x2 >> PREC_SHIFT == 1); // state is normalized
        self.io.flush(self.x1)
    }
}

pub struct ArithmeticDecoder<R> {
    x1: u32, // low
    x2: u32, // range
    x:  u32, // state
    io: ACReader<R>
}

impl<R: Read> ArithmeticDecoder<R> {
    pub fn new(reader: R) -> io::Result<Self> {
        let mut io = ACReader::new(reader);
        let x = io.read_u32()?;
        Ok(Self { io, x, x1: 0, x2: u32::MAX })
    }
}

impl<R: Read> ACDecoder for ArithmeticDecoder<R> {
    fn decode(&mut self, prob: u16) -> io::Result<u8> {
        let xmid = lerp(self.x1, self.x2, prob);
        let bit = (self.x <= xmid).into();

        // Update range (kinda like binary search)
        match bit {
            1 => self.x2 = xmid,
            _ => self.x1 = xmid + 1
        }

        // Renormalize range -> write matching bits to stream
        while ((self.x1 ^ self.x2) >> PREC_SHIFT) == 0 {
            self.x = (self.x << 1) | u32::from(self.io.read_bit()?);
            self.x1 <<= 1;
            self.x2 = (self.x2 << 1) | 1;
        }

        // E3 renorm (special case) -> increase parity bits but don't write anything to stream
        while self.x1 >= Q1 && self.x2 < Q3 {
            self.x1 = (self.x1 << 1) & RLOW_MOD;
            self.x2 = (self.x2 << 1) | RHIGH_MOD;
            self.x = ((self.x << 1) ^ RMID) | u32::from(self.io.read_bit()?);
        }

        Ok(bit)
    }
}

#[inline(always)]
fn lerp(x1: u32, x2: u32, prob: u16) -> u32 {
    const P_SHIFT: u32 = u32::BITS - u16::BITS;
    let range = u64::from(x2 - x1);
    let p = if prob != 0 { u64::from(prob) << P_SHIFT } else { 1 };
    let lerped_range = (range * p) >> (P_SHIFT + u16::BITS);
    let xmid = x1 + u32::try_from(lerped_range).unwrap(); // should never fail, as both range and p < 2^32
    debug_assert!(xmid >= x1 && xmid < x2);
    xmid
}
