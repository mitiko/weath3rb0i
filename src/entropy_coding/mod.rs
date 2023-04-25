pub mod ac_io;
use std::io;

const PREC_SHIFT: u32 = u32::BITS - 1; // 31
const Q1: u32 = 1 << (PREC_SHIFT - 1); // 0x40000000, 1 = 0b01, quarter 1
const Q2: u32 = 2 << (PREC_SHIFT - 1); // 0x80000000, 2 = 0b10, range middle
const Q3: u32 = 3 << (PREC_SHIFT - 1); // 0xC0000000, 3 = 0b11, quarter 3
const RLO_MOD: u32 = (1 << PREC_SHIFT) - 1; // 0x7FFFFFFF, range low modify
const RHI_MOD: u32 = (1 << PREC_SHIFT) + 1; // 0x80000001, range high modify

/// The `ArithmeticCoder` encodes/decodes bits given a probability
pub struct ArithmeticCoder<T> {
    x1: u32, // low
    x2: u32, // high
    x: u32,  // state
    io: T,   // bit reader/writer
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

impl<W: ACWrite> ArithmeticCoder<W> {
    pub fn new_coder(writer: W) -> Self {
        Self { io: writer, x1: 0, x2: u32::MAX, x: 0 }
    }

    /// TODO: Encode 4-bits at once.
    pub fn encode4(&mut self, nib: u8, probs: [u16; 4]) -> io::Result<()> {
        // todo!();
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
            self.x1 = (self.x1 << 1) & RLO_MOD;
            self.x2 = (self.x2 << 1) | RHI_MOD;
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
            self.x1 = (self.x1 << 1) & RLO_MOD;
            self.x2 = (self.x2 << 1) | RHI_MOD;
            self.x = ((self.x << 1) ^ Q2) | u32::from(self.io.read_bit()?);
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
    let xmid = x1 + u32::try_from(lerped_range).unwrap();
    debug_assert!(xmid >= x1 && xmid < x2);
    xmid
}

#[cfg(test)]
mod tests {
    use std::{borrow::Borrow, fs::File, io::{BufWriter, Read}};

    use super::{
        ac_io::{ACReader, ACWriter},
        ACRead, ACWrite, ArithmeticCoder,
    };

    fn compress(filename: &str, input: &[u8], probabilities: &[u16]) -> Vec<u8> {
        let file = File::create(filename).unwrap();
        let writer = ACWriter::new(BufWriter::new(file));
        let mut ac = ArithmeticCoder::new_coder(writer);

        let mut reader = ACReader::new(input);
        for &prob in probabilities {
            let bit = reader.read_bit().unwrap();
            ac.encode(bit, prob).unwrap();
        }

        ac.flush().unwrap();
        let compressed = std::fs::read(filename).unwrap();
        std::fs::remove_file(filename).unwrap();
        compressed
    }

    fn decompress(filename: &str, input: &[u8], probabilities: &[u16]) -> Vec<u8> {
        let reader = ACReader::new(input);
        let mut ac = ArithmeticCoder::new_decoder(reader).unwrap();

        let file = File::create(filename).unwrap();
        let mut writer = ACWriter::new(BufWriter::new(file));
        for &prob in probabilities {
            let bit = ac.decode(prob).unwrap();
            writer.write_bit(bit).unwrap();
        }

        // flush always appends \x00 to end, because it's aligned
        writer.flush(0).unwrap();
        let mut decompressed = std::fs::read(filename).unwrap();
        assert_eq!(decompressed.pop(), Some(0x00));
        std::fs::remove_file(filename).unwrap();
        decompressed
    }

    #[test]
    fn best_model_zeroes() {
        let block_size = 1 << 15;
        let input = [0x00].repeat(block_size);
        let probabilities = [0].repeat(block_size * 8);
        let compressed = compress("best_model_zeroes.bin", &input, &probabilities);
        let decompressed = decompress("best_model_zeroes.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        // only 0s will always compress to 0 bytes (and flush will add 1)
        assert_eq!(compressed.len(), 1);
    }

    #[test]
    fn best_model_ones() {
        let block_size = 1 << 15;
        let input = [0xff].repeat(block_size);
        let probabilities = [u16::MAX].repeat(block_size * 8);
        let compressed = compress("best_model_ones.bin", &input, &probabilities);
        let decompressed = decompress("best_model_ones.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        assert_eq!(compressed.len(), 1);

        let block_size = 1 << 16;
        let input = [0xff].repeat(block_size);
        let probabilities = [u16::MAX].repeat(block_size * 8);
        let compressed = compress("best_model_ones.bin", &input, &probabilities);
        let decompressed = decompress("best_model_ones.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        // loss is 2^-16 bits/bit, so at 2^16 bytes, we get an extra byte
        assert_eq!(compressed.len(), 2);
    }

    #[test]
    fn best_model_alternating() {
        let block_size = 1024;
        let input = [0x55].repeat(block_size);
        let probabilities = [0, u16::MAX].repeat(block_size * 8 / 2);
        let compressed = compress("best_model_alternating.bin", &input, &probabilities);
        let decompressed = decompress("best_model_alternating.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        assert_eq!(compressed.len(), 1);
    }

    #[test]
    fn worst_model_zeroes() {
        let block_size = 16;
        let input = [0x00].repeat(block_size);
        let probabilities = [u16::MAX].repeat(block_size * 8);
        let compressed = compress("worst_model_zeroes.bin", &input, &probabilities);
        let decompressed = decompress("worst_model_zeroes.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        // loss is (1 - 2^-16) = (2^16 - 1)/(2^16) bits/bit
        assert_eq!(compressed.len(), 16 * block_size + 1);
    }

    #[test]
    fn worst_model_ones() {
        let block_size = 16;
        let input = [0xff].repeat(block_size);
        let probabilities = [0].repeat(block_size * 8);
        let compressed = compress("worst_model_ones.bin", &input, &probabilities);
        let decompressed = decompress("worst_model_ones.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        // TODO: Optimize zeroes bias
        // practical loss is a little more than (1 - 2^-16) bits/bit
        assert_eq!(compressed.len(), 32 * block_size + 1);
    }

    #[test]
    fn worst_model_alternating() {
        let block_size = 16;
        let input = [0x55].repeat(block_size);
        let probabilities = [u16::MAX, 0].repeat(block_size * 8 / 2);
        let compressed = compress("worst_model_alternating.bin", &input, &probabilities);
        let decompressed = decompress("worst_model_alternating.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        // TODO: Optimize zeroes bias
        // half the time it gets the bias, half the time it does not
        // 24 = (32 + 16) / 2
        assert_eq!(compressed.len(), 24 * block_size + 1);
    }

    #[test]
    fn no_model() {
        let block_size = 128;
        let input = [0xaa, 0x55].repeat(block_size / 2);
        let probabilities = [1 << 15].repeat(block_size * 8);
        let compressed = compress("no_model.bin", &input, &probabilities);
        let decompressed = decompress("no_model.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        // flush always adds one byte to aligned sequences
        assert!(compressed.len() == block_size + 1);
    }

    #[test]
    fn half_good_model() {
        let block_size = 128;
        let input = [0x55].repeat(block_size);
        let probabilities = [1 << 15, u16::MAX].repeat(block_size * 8 / 2);
        let compressed = compress("half_good_model.bin", &input, &probabilities);
        let decompressed = decompress("half_good_model.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        // cross entropy is 1/2 * 1 + 1/2 * 2^-16 ~ 1/2
        assert_eq!(compressed.len(), block_size / 2);
    }

    #[test]
    fn half_bad_model() {
        let block_size = 128;
        let input = [0x55].repeat(block_size);
        let probabilities = [1 << 15, 0].repeat(block_size * 8 / 2);
        let compressed = compress("half_bad_model.bin", &input, &probabilities);
        let decompressed = decompress("half_bad_model.tmp", &compressed, &probabilities);
        assert_eq!(input, decompressed);
        // half the time it gets a 16x penalty, half the time it writes the bit
        // flush always adds one byte to aligned sequences
        assert_eq!(compressed.len(), 16 * block_size + block_size / 2 + 1);
    }

    // TODO: Fuzzing tests
}
