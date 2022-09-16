/*!
A 32-bit high precision binary arithmetic coder.

It uses a 32-bit range and state and encodes 16-bit probabilities.
The probabilities are renormalized to 64-bit and the range update is performed in 64-bit math so no overflows can happen
Then they're shifted back to 32-bits

# Examples

Initialize and run encoder:
```
let mut writer = BufWriter::new(File::create(output_file)?);
let reader = {
    let f = File::open(input_file)?;
    let len = f.metadata()?.len();

    // Write len to file
    // The decoder will need this information to know how many bytes to consume
    writer.write_all(&len.to_be_bytes())?;
    BufReader::new(f)
};
// Initialize the encoder
let mut ac = ArithmeticCoder::<_, BufReader<File>>::init_enc(writer);
let mut model = init_model();

for byte_res in reader.bytes() {
    let byte = byte_res?;
    // For each nibble in byte
    for nib in [byte >> 4, byte & 15] {
        let probabilities = model.predict4(nib); // [u8; 4]
        model.update4(nib);
        // Encode a nibble with 4 probabilities for each bit
        ac.encode4(nib, probabilities);
    }
}

// Don't forget to flush the encoder!
ac.flush();
Ok(())
```

Initialize and run the decoder:
```
let mut writer = {
    let buf_writer = BufWriter::new(File::create(output_file)?);
    BitWriter::new(buf_writer)
};

// Read len from header of file
let len = {
    let mut len_buf = [0; std::mem::size_of::<u64>()];
    reader.read_exact(&mut len_buf)?;
    u64::from_be_bytes(len_buf)
};
// Initialize the decoder
let mut ac = ArithmeticCoder::<BufWriter<File>, _>::init_dec(reader);
let mut model = init_model();

// Write exactly len bytes to decompressed file
for _ in 0..len {
    // For each bit in byte
    for _ in 0..8 {
        let p = model.predict();
        let bit = ac.decode(p);
        writer.write_bit(bit);
        model.update(bit);
    }
}

// Don't forget to flush the writer
writer.try_flush();
Ok(())
```
*/

#![deny(missing_docs)]

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

/// A 32-bit high precision binary arithmetic coder.
/// 
/// Allows encoding bits with non-uniform 16-bit probability.  
/// It uses the `std::io::{Read, Write}` traits to be as generic as possible,
/// and until #48331 gets merged, it relies on std.
pub struct ArithmeticCoder<TWrite: Write, TRead: Read> {
    /// Low of range
    x1: u32,
    /// High of range
    x2: u32,
    /// State (decode only)
    x: u32,
    /// Arithmetic coding specific IO
    io: ArithmeticCoderIO<TWrite, TRead>
}

impl<TWrite, TRead> ArithmeticCoder<TWrite, TRead>
where TWrite: Write, TRead: Read {
    /// Initialize an encoder with a stream to write to.
    /// 
    /// Example:
    /// ```
    /// let mut writer = BufWriter::new(File::create(out_file)?);
    /// let mut ac = ArithmeticCoder::<_, BufReader<File> /* any reader */>::init_enc(writer);
    pub fn init_enc(stream: TWrite) -> Self {
        Self {
            x1: 0, x2: u32::MAX, x: u32::default(),
            io: Encode(ACWriter::new(stream))
        }
    }
    
    /// Initialize a decoder with a stream to read from.
    /// 
    /// Reads 4 bytes BE to fill the initial state.
    /// If the stream contains less than 4 bytes, the rest is assumed to be zeroes.
    ///
    /// Example:
    /// ```
    /// let mut reader = BufReader::new(File::open(input_file)?);
    /// let mut ac = ArithmeticCoder::<_, BufWriter<File> /* any writer */>::init_dec(reader);
    /// ```
    pub fn init_dec(mut stream: TRead) -> Self {
        let x = {
            let mut buf = [0; std::mem::size_of::<u32>()];
            stream.read_exact(&mut buf).unwrap(); // if we reach EOF, we're just padding with 0s anyway
            u32::from_be_bytes(buf)
        };

        Self {
            x1: 0, x2: u32::MAX, x,
            io: Decode(ACReader::new(stream))
        }
    }

    /// Encodes a nibble (4 bits) to the stream.
    /// 
    /// This is an optimization to the encoder as the hashtable design encourages it.
    /// It allows the encoder to inline better.
    /// 
    /// In future version it's possible to perform 128-bit math and do a single renormalization,
    /// essentially eliminating a dependency chain with math and two more branches.
    /// 
    /// Example:
    /// ```
    /// for nib in [byte >> 4, byte & 15] {
    ///     let probs4 = model.predict4(nib); // [u8; 4] 
    ///     model.update4(nib);
    ///     ac.encode4(nib, probs4);
    /// }
    /// ```
    /// 
    /// 
    #[inline(never)]
    pub fn encode4(&mut self, nib: u8, probs: [u16; 4]) {
        // TODO: Optimize to a single update of ranges?
        // TODO: is this more clear than a for loop with (nib >> (3-i)) & 1 and probs[i]?
        self.encode(nib >> 3, probs[0]);
        self.encode((nib >> 2) & 1, probs[1]);
        self.encode((nib >> 1) & 1, probs[2]);
        self.encode(nib & 1, probs[3]);
    }

    /// Encodes a bit to the stream
    fn encode(&mut self, bit: u8, prob: u16) {
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

    /// Decodes a bit from the stream.
    /// 
    /// NOTE: Calls to decode will panic if EOF is reached.
    /// This is intentional, the callers of decode must know in advance how long the decompressed stream must be.
    /// 
    /// Example:
    /// ```
    /// for _ in 0..len {
    ///     for _ in 0..u8::BITS {
    ///         let p = model.predict();
    ///         let bit = ac.decode(p);
    /// 
    ///         writer.write_bit(bit);
    ///         model.update(bit); // call to model last, so the compiler can do some optimizations on loop unroll
    ///     }
    /// }
    /// ```
    #[inline(never)]
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

    /// Return the lerp-ed middle of the range.
    fn get_mid(&self, prob: u16) -> u32 {
        let range = u64::from(self.x2 - self.x1);
        let prob = renorm_prob(prob);
        let lerped_range = (range * prob) >> (u64::BITS - u32::BITS);
        let xmid = self.x1 + lerped_range as u32;
        debug_assert!(xmid >= self.x1 && xmid < self.x2);
        xmid
    }

    /// Flushes the encoder and also the internal stream.
    /// 
    /// It pads the last byte with bits from the MSBs of the range's low.
    pub fn flush(&mut self) {
        let w = self.io.as_enc();
        debug_assert!(self.x1 >> PREC_SHIFT == 0 && self.x2 >> PREC_SHIFT == 1);

        w.write_bit(1);
        w.flush(self.x1 >> (u32::BITS - u8::BITS));
    }
}

/// Renormalizes the probability to a 64-bit representation.
fn renorm_prob(prob: u16) -> u64 {
    let mut prob = u64::from(prob) << (u32::BITS - u16::BITS);
    if prob == 0 {
        prob = 1;
    }
    
    debug_assert!(prob > 0 && prob < u64::from(u32::MAX));
    prob
}

/// Arithmetic coder specific IO
/// 
/// Contains wrappers around `bit_helpers::{BitReader, BitWriter}`
// TODO: Add BufferedBit{Reader,Writer}
mod arithmetic_coder_io {
    #![deny(clippy::missing_docs_in_private_items)]

    use std::{io::{Write, Read}, convert::TryInto};
    use crate::bit_helpers::{BitWriter, BitReader};
    pub use ArithmeticCoderIO::{Encode, Decode};

    /// ArithmeticCoderIO is an invariant of read or write.
    /// The coder is either in encode mode (writing to stream) or decode mode (reading from stream).
    pub enum ArithmeticCoderIO<TWrite: Write, TRead: Read> {
        /// In encode mode we write bits
        Encode(ACWriter<TWrite>),
        /// In decode mode we read bits
        Decode(ACReader<TRead>)
    }

    impl<TWrite, TRead> ArithmeticCoderIO<TWrite, TRead>
    where TWrite: Write, TRead: Read {
        /// Returns the contained [`Decode`] value, consuming the `self` value.
        /// 
        /// The encoder should not try to decode in encode mode.
        /// The `debug_unreachable!` macro is called when the wrong mode is used
        /// It panics in debug mode and inserts an intrinsic for the compiler to optimize in release
        /// If the wrong mode is used in release, this is Undefined Behaviour
        pub fn as_dec(&mut self) -> &mut ACReader<TRead> {
            match self {
                Decode(r) => r,
                Encode(_) => unsafe { debug_unreachable!("[AC] Tried to use reader in encode mode"); }
            }
        }
        
        /// Returns the contained [`Encode`] value, consuming the `self` value.
        /// 
        /// The encoder should not try to encode in decode mode.
        /// The `debug_unreachable!` macro is called when the wrong mode is used
        /// It panics in debug mode and inserts an intrinsic for the compiler to optimize in release
        /// If the wrong mode is used in release, this is Undefined Behaviour
        pub fn as_enc(&mut self) -> &mut ACWriter<TWrite> {
            match self {
                Encode(w) => w,
                Decode(_) => unsafe { debug_unreachable!("[AC] Tried to use writer in decode mode") },
            }
        }
    }

    /// The `ACReader` is a wrapper around `bit_helpers::BitReader`
    pub struct ACReader<TRead: Read> {
        /// The internal (bit) reader
        reader: BitReader<TRead>
    }
    
    impl<TRead: Read> ACReader<TRead> {
        /// Initialize from a stream
        pub fn new(stream: TRead) -> Self {
            Self { reader: BitReader::new(stream) }
        }
        
        /// Read bit (or 0 on EOF) and bit extend to u32 
        pub fn read_bit(&mut self) -> u32 {
            self.reader.read_bit().unwrap_or(0).into()
        }
    }
    
    /// The `ACWriter` is a wrapper around `bit_helpers::BitWriter`
    pub struct ACWriter<TWrite: Write> {
        /// The internal (bit) writer
        writer: BitWriter<TWrite>,
        /// Parity bits to write - from E3 mappings
        rev_bits: u64
    }

    impl<TWrite: Write> ACWriter<TWrite> {
        /// Initialize from a stream
        pub fn new(stream: TWrite) -> Self {
            Self { writer: BitWriter::new(stream), rev_bits: 0 }
        }

        /// Write bit and potentially parity (reverse) bits
        pub fn write_bit(&mut self, bit: u32) {
            let bit = bit.try_into().unwrap();
            self.writer.write_bit(bit);
    
            while self.rev_bits > 0 {
                self.writer.write_bit(bit ^ 1);
                self.rev_bits -= 1;
            }
        }

        /// Increases the parity bits to write (on E3 mappings)
        pub fn inc_parity(&mut self) {
            self.rev_bits += 1;
        }

        /// Flushes the internal (bit) writer with a byte to pad the remaining bits
        pub fn flush(&mut self, pad_byte: u32) {
            let pad_byte = pad_byte.try_into().unwrap();
            self.writer.flush(pad_byte);
        }
    }
}
