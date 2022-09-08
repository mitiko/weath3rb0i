use std::io::{Read, Write};
use std::fs::File;
use ACBitBuf::{Encode, Decode};

const PRECISION:  u32 = u32::BITS;            // 32
const PREC_SHIFT: u32 = PRECISION - 1;        // 31
const Q1:   u32 = 1 << (PRECISION - 2);       // 0x40000000, 1 = 0b01
const RMID: u32 = 2 << (PRECISION - 2);       // 0x80000000, 2 = 0b10
const Q3:   u32 = 3 << (PRECISION - 2);       // 0xC0000000, 3 = 0b11
const RLOW_MOD:  u32 = (1 << PREC_SHIFT) - 1; // 0x7FFFFFFF, AND with, set high bit 0, keep low bit (to 0 after shift)
const RHIGH_MOD: u32 = (1 << PREC_SHIFT) + 1; // 0x80000001, OR with, set high bit 1, set low bit 1


pub struct ArithmeticCoder<TW: Write, TR: Read> {
    x1: u32, // low
    x2: u32, // high
    x: u32,  // state
    buf: ACBitBuf<TW, TR>
}

impl<TW: Write, TR: Read> ArithmeticCoder<TW, TR> {
    pub fn init_enc(stream: TW) -> Self {
        Self {
            x1: 0, x2: u32::MAX, x: 0,
            buf: ACBitBuf::Encode(BitWriter::new(stream))
        }
    }

    pub fn init_dec(stream: TR) -> Self {
        let mut x = 0;
        let mut bit_reader = BitReader::new(stream);
        for _ in 0..32 {
            x = (x << 1) | bit_reader.bit_read().expect("[AC] Couldn't read initial 4 bytes") as u32;
        }

        Self {
            x1: 0, x2: u32::MAX, x,
            buf: ACBitBuf::Decode(bit_reader)
        }
    }


    fn process(&mut self, bit: Option<u8>, mut prob: u16) -> Result<u8, EOF> {
        if prob == 0 { prob += 1; }
        debug_assert!(prob > 0 && prob < u16::MAX);

        let xmid = self.x1 + (((self.x2 - self.x1) as u64 * prob as u64) >> u16::BITS) as u32;
        let bit = bit.unwrap_or((self.x <= xmid).into());
        debug_assert!(xmid >= self.x1 && xmid < self.x2);

        match bit {
            1 => self.x2 = xmid,
            _ => self.x1 = xmid + 1
        }

        while ((self.x1 ^ self.x2) >> PREC_SHIFT) == 0 {  // pass equal leading bits of range
            match &mut self.buf {
                Encode(w) => w.bit_write((self.x2 >> PREC_SHIFT) as u8),
                Decode(r) => self.x = (self.x << 1) | r.bit_read()? as u32,
            }
            self.x1 <<= 1;
            self.x2 = (self.x2 << 1) | 1;
        }

        while self.x1 >= Q1 && self.x2 < Q3 {
            match &mut self.buf {
                Encode(w) => w.inc_parity(),
                Decode(r) => self.x = ((self.x << 1) ^ RMID) + r.bit_read()? as u32,
            }
            self.x1 = (self.x1 << 1) & RLOW_MOD;
            self.x2 = (self.x2 << 1) | RHIGH_MOD;
        }

        Ok(bit)
    }

    pub fn encode4(&mut self, nib: u8, probs: [u16; 4]) {
        // TODO: Optimize to a single update of ranges?
        // TODO: is this more clear than a for loop with (nib >> (3-i)) & 1 and probs[i]?
        let _ = self.process(Some(nib >> 3), probs[0]);
        let _ = self.process(Some((nib >> 2) & 1), probs[1]);
        let _ = self.process(Some((nib >> 1) & 1), probs[2]);
        let _ = self.process(Some(nib & 1), probs[3]);
    }

    pub fn decode(&mut self, prob: u16) -> Result<u8, EOF> {
        self.process(None, prob)
    }

    pub fn flush(&mut self) {
        match &mut self.buf {
            Encode(w) => {
                for _ in 0..32 {
                    w.bit_write((self.x1 >> PREC_SHIFT) as u8);
                    self.x1 <<= 1;
                }
                w.flush();
            }
            _ => { panic!("[AC] Tried to flush in decode mode"); }
        }
    }
}

enum ACBitBuf<TW: Write, TR: Read> {
    Encode(BitWriter<TW>),
    Decode(BitReader<TR>)
}

// TODO: Make this private (only for AC to use)
pub struct BitWriter<TW: Write> {
    t: u8,
    count: u8,
    reverse_bits: u32,
    stream: TW
}

impl<TW: Write> BitWriter<TW> {
    pub fn new(stream: TW) -> Self {
        Self { t: 0, count: 0, reverse_bits: 0, stream }
    }

    pub fn bit_write_raw(&mut self, bit: u8) {
        self.t = (self.t << 1) | bit;
        self.count += 1;

        if self.count == 8 {
            if self.stream.write(&[self.t]).unwrap_or(0) == 0 {
                panic!("[AC] Write failed!");
            }

            self.t = 0;
            self.count = 0;
        }
    }

    fn bit_write(&mut self, bit: u8) {
        self.bit_write_raw(bit);

        while self.reverse_bits > 0 {
            self.bit_write_raw(bit ^ 1);
            self.reverse_bits -= 1;
        }
    }

    fn inc_parity(&mut self) {
        self.reverse_bits += 1;
    }

    pub fn flush(&mut self) {
        println!("FLUSH");
        dbg!(self.count);
        dbg!(self.t);

        while self.count > 0 {
            self.bit_write_raw(0);
        }

        debug_assert!(self.count == 0 && self.t == 0);
        self.stream.flush().expect("[AC] Couldn't flush!");
    }
}

struct BitReader<TR: Read> {
    t: u8,
    count: u8,
    byte_buf: [u8; 1],
    stream: TR
}

#[derive(Debug)]
pub struct EOF;

impl<TR: Read> BitReader<TR> {
    fn new(stream: TR) -> Self {
        Self { t: 0, count: 0, byte_buf: [0], stream }
    }

    fn bit_read(&mut self) -> Result<u8, EOF> {
        if self.count == 0 {
            self.count = 8;

            let bytes_read = self.stream.read(&mut self.byte_buf).unwrap_or(usize::MAX);
            match bytes_read {
                1 => self.t = self.byte_buf[0], // all fine
                0 => return Err(EOF),
                _ => panic!("[AC] Read failed!")
            }
        }
        
        self.count -= 1;
        Ok((self.t >> self.count) & 1)
    }
}
