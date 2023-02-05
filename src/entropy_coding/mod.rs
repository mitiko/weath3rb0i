pub mod ac_io;
use std::{io::{Read, Write, self}, fmt::LowerExp};
use self::ac_io::{ACRead, ACReader, ACWrite, ACWriter};

const PRECISION:  u32 = u32::BITS;            // 32
const PREC_SHIFT: u32 = PRECISION - 1;        // 31
const Q1:   u32 = 1 << (PRECISION - 2);       // 0x40000000, 1 = 0b01, quarter 1
const RMID: u32 = 2 << (PRECISION - 2);       // 0x80000000, 2 = 0b10, range mid
const Q3:   u32 = 3 << (PRECISION - 2);       // 0xC0000000, 3 = 0b11, quarter 3
const RLOW_MOD:  u32 = (1 << PREC_SHIFT) - 1; // 0x7FFFFFFF, range low modifier, AND with -> sets high bit to 0, keeps low bit (to 0 after shift)
const RHIGH_MOD: u32 = (1 << PREC_SHIFT) + 1; // 0x80000001, range high modifier, OR with -> sets high bit to 1, sets low bit to 1


/// The `ArithmeticCoder` compresses bits given a probability and writes
/// fractional bits to an internal `Write` instance.
///
/// See https://en.wikipedia.org/wiki/Arithmetic_coding
pub struct ArithmeticCoder<T> {
    x1: u32, // low
    x2: u32, // high
    x: u32,  // state
    io: T    // bit reader/writer
}

impl<W: ACWrite> ArithmeticCoder<W> {
    pub fn new_coder(writer: W) -> Self {
        Self { io: writer, x1: 0, x2: u32::MAX, x: 0 }
    }

    /// Encodes 4-bits at once.
    pub fn encode4(&mut self, nib: u8, probs: [u16; 4]) -> io::Result<()> {
        self.encode(nib & 0b1000, probs[0])?;
        self.encode(nib & 0b0100, probs[1])?;
        self.encode(nib & 0b0010, probs[2])?;
        self.encode(nib & 0b0001, probs[3])?;
        Ok(())
    }

    pub fn encode(&mut self, bit: u8, prob: u16) -> io::Result<()> {
        let xmid = lerp(self.x1, self.x2, prob);

        // Update range (kinda like binary search)
        match bit {
            0 => self.x1 = xmid + 1,
            _ => self.x2 = xmid,
        }

        // Renormalize range -> write matching bits to stream
        while ((self.x1 ^ self.x2) >> PREC_SHIFT) == 0 {
            self.io.write_bit(self.x1 >> PREC_SHIFT)?;
            self.x1 <<= 1;
            self.x2 = (self.x2 << 1) | 1;
        }

        // E3 renorm (special case) -> increase parity
        while self.x1 >= Q1 && self.x2 < Q3 {
            self.io.inc_parity();
            self.x1 = (self.x1 << 1) & RLOW_MOD;
            self.x2 = (self.x2 << 1) | RHIGH_MOD;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        // assert state is normalized
        debug_assert!(self.x1 >> PREC_SHIFT == 0 && self.x2 >> PREC_SHIFT == 1);
        self.io.flush(self.x2)
    }
}

impl<R: ACRead> ArithmeticCoder<R> {
    pub fn new_decoder(mut reader: R) -> io::Result<Self> {
        let x = reader.read_u32()?;
        Ok(Self { io: reader, x1: 0, x2: u32::MAX, x })
    }

    pub fn decode(&mut self, prob: u16) -> io::Result<u8> {
        let xmid = lerp(self.x1, self.x2, prob);
        let bit = (self.x <= xmid).into();

        // Update range (kinda like binary search)
        match bit {
            0 => self.x1 = xmid + 1,
            _ => self.x2 = xmid,
        }

        // Renormalize range -> read new bits from stream
        while ((self.x1 ^ self.x2) >> PREC_SHIFT) == 0 {
            self.x1 <<= 1;
            self.x2 = (self.x2 << 1) | 1;
            self.x = (self.x << 1) | u32::from(self.io.read_bit()?);
        }

        // E3 renorm (special case) -> fix parity
        while self.x1 >= Q1 && self.x2 < Q3 {
            self.x1 = (self.x1 << 1) & RLOW_MOD;
            self.x2 = (self.x2 << 1) | RHIGH_MOD;
            self.x = ((self.x << 1) ^ RMID) | u32::from(self.io.read_bit()?);
        }

        Ok(bit)
    }
}

// TODO: Verify rounded range, test speed
// let lerped_range = (range * p) >> (RANGE_SHIFT - 1);
// let rounded_range = (lerped_range >> 1) + (lerped_range & 1);
// let xmid = x1 + u32::try_from(rounded_range).unwrap();
#[inline(always)]
fn lerp(x1: u32, x2: u32, prob: u16) -> u32 {
    const P_SHIFT: u32 = u32::BITS - u16::BITS;
    const RANGE_SHIFT: u32 = P_SHIFT + u16::BITS;

    let mut p = u64::from(prob) << P_SHIFT;
    if p == 0 { p = 1; }

    let range = u64::from(x2 - x1);
    let lerped_range = (range * p) >> RANGE_SHIFT;

    // never fails, as both range and p < 2^32
    let xmid = x1 + u32::try_from(lerped_range).unwrap();

    debug_assert!(xmid >= x1 && xmid < x2);
    xmid
}
