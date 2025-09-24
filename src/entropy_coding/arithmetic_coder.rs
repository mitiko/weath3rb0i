use crate::u32;
use std::{io, marker::PhantomData};

const PREC_SHIFT: u32 = u32::BITS - 1; // 31
const Q1: u32 = 1 << (PREC_SHIFT - 1); // 0x40000000, 1 = 0b01, quarter 1
const Q2: u32 = 2 << (PREC_SHIFT - 1); // 0x80000000, 2 = 0b10, range middle
const Q3: u32 = 3 << (PREC_SHIFT - 1); // 0xC0000000, 3 = 0b11, quarter 3
const RLO_MOD: u32 = (1 << PREC_SHIFT) - 1; // 0x7FFFFFFF, range low modify
const RHI_MOD: u32 = (1 << PREC_SHIFT) + 1; // 0x80000001, range high modify

/// The `ArithmeticCoder` encodes/decodes bits given a probability
#[derive(Clone)]
pub struct ArithmeticCoder<T> {
    x1: u32,                 // low
    x2: u32,                 // high
    x: u32,                  // state
    _marker: PhantomData<T>, // use for io
}

pub trait ACRead {
    /// Read bit or 0 on EOF
    fn read_bit(&mut self) -> io::Result<u8>;
    /// Read 4 bytes BE as u32 and pad with 0s on EOF
    fn read_u32(&mut self) -> io::Result<u32>;
}

pub trait ACWrite {
    /// Increases the number of reverse bits to write
    fn inc_parity(&mut self);
    /// Writes a bit and maintains E3 mapping logic
    fn write_bit(&mut self, bit: impl TryInto<u8>) -> io::Result<()>;
    /// Flushes leftover parity bits and internal writer
    fn flush(&mut self, padding: u32) -> io::Result<()>;
}

// TODO: move the W: ACWrite restriction to function, so ArithmeticCoder is not generic
impl<W: ACWrite> ArithmeticCoder<W> {
    pub fn new_coder() -> Self {
        Self { x1: 0, x2: u32::MAX, x: 0, _marker: PhantomData }
    }

    pub fn encode(&mut self, bit: u8, prob: u16, io: &mut W) -> io::Result<()> {
        let xmid = lerp(self.x1, self.x2, prob);

        // Update range (kinda like binary search)
        match bit {
            0 => self.x1 = xmid + 1,
            _ => self.x2 = xmid,
        }

        // Renormalize range -> write matching bits to stream
        while ((self.x1 ^ self.x2) >> PREC_SHIFT) == 0 {
            io.write_bit(self.x1 >> PREC_SHIFT)?;
            self.x1 <<= 1;
            self.x2 = (self.x2 << 1) | 1;
        }

        // E3 renorm (special case) -> increase parity
        while self.x1 >= Q1 && self.x2 < Q3 {
            io.inc_parity();
            self.x1 = (self.x1 << 1) & RLO_MOD;
            self.x2 = (self.x2 << 1) | RHI_MOD;
        }

        Ok(())
    }

    pub fn flush(&mut self, io: &mut W) -> io::Result<()> {
        // assert state is normalized
        debug_assert!(self.x1 >> PREC_SHIFT == 0 && self.x2 >> PREC_SHIFT == 1);
        io.flush(self.x2)
    }
}

impl<R: ACRead> ArithmeticCoder<R> {
    pub fn new_decoder(reader: &mut R) -> io::Result<Self> {
        let x = reader.read_u32()?;
        Ok(Self { x1: 0, x2: u32::MAX, x, _marker: PhantomData })
    }

    pub fn decode(&mut self, prob: u16, io: &mut R) -> io::Result<u8> {
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
            self.x = (self.x << 1) | u32::from(io.read_bit()?);
        }

        // E3 renorm (special case) -> fix parity
        while self.x1 >= Q1 && self.x2 < Q3 {
            self.x1 = (self.x1 << 1) & RLO_MOD;
            self.x2 = (self.x2 << 1) | RHI_MOD;
            self.x = ((self.x << 1) ^ Q2) | u32::from(io.read_bit()?);
        }

        Ok(bit)
    }
}

#[inline(always)]
fn lerp(x1: u32, x2: u32, prob: u16) -> u32 {
    // make prob 32-bit & always leave chance
    let p = if prob == 0 { 1 } else { u64::from(prob) << 16 };
    let range = u64::from(x2 - x1);
    let lerped_range = (range * p) >> 32;

    // no overflows/underflows, as both range < 2^32 and p < 2^32
    let xmid = x1 + u32!(lerped_range);
    debug_assert!(xmid >= x1 && xmid < x2);
    xmid
}
