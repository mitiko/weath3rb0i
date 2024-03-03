use core::slice::from_mut as into_slice;
use std::io::{self, ErrorKind, Read, Write};

use super::arithmetic_coder::{ACRead, ACWrite};

/// Arithmetic coder read io for `io::Read` types
pub struct ACReader<R> {
    inner: R,
    buf: u8,
    mask: u8,
}

impl<R: Read> ACReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner, buf: 0, mask: 0 }
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        debug_assert!(self.mask == 0);
        let mut byte = 0;
        let result = self.inner.read_exact(into_slice(&mut byte));

        match result {
            Err(err) if err.kind() == ErrorKind::UnexpectedEof => Ok(0),
            _ => result.map(|_| byte),
        }
    }
}

impl<R: Read> ACRead for ACReader<R> {
    fn read_bit(&mut self) -> io::Result<u8> {
        self.mask >>= 1; // move to next bit
        if self.mask == 0 {
            self.buf = self.read_byte()?; // fill
            self.mask = 1 << 7; // then move to first bit
        }
        Ok((self.buf & self.mask > 0).into())
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        let bytes = [
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ];
        Ok(u32::from_be_bytes(bytes))
    }
}

/// Arithmetic coder write io for `io::Write` types
pub struct ACWriter<W> {
    inner: W,
    buf: u8,
    idx: u8,
    rev_bits: u64,
}

impl<W: Write> ACWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, buf: 0, idx: 0, rev_bits: 0 }
    }
}

impl<W: Write> ACWrite for ACWriter<W> {
    fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    fn write_bit(&mut self, bit: impl TryInto<u8>) -> io::Result<()> {
        let bit: u8 = bit.try_into().unwrap_or_default();
        debug_assert!(bit <= 1, "Tried to write invalid bit");

        let mut write_bit_raw = |bit: u8| -> io::Result<()> {
            self.buf = (self.buf << 1) | bit;
            self.idx = (self.idx + 1) % 8;
            if self.idx == 0 {
                self.inner.write_all(&[self.buf])?
            }
            Ok(())
        };

        write_bit_raw(bit)?;
        while self.rev_bits > 0 {
            self.rev_bits -= 1;
            write_bit_raw(bit ^ 1)?;
        }
        Ok(())
    }

    fn flush(&mut self, mut state: u32) -> io::Result<()> {
        self.write_bit(state >> 31)?;
        state <<= 1;
        while self.idx > 0 {
            self.write_bit(state >> 31)?;
            state <<= 1;
        }
        self.inner.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ACRead, ACReader, ACWrite, ACWriter};

    #[test]
    fn read_bits() {
        let data: [u8; 2] = [0b0101_0101, 0b1010_1010];
        let mut reader = ACReader::new(data.as_ref());
        let truth = [0, 1]
            .iter()
            .cycle()
            .take(17) // 16 bits, but eliminate 1
            .enumerate()
            .filter(|&(i, _)| i != 8) // fancy
            .map(|(_, x)| x);

        // assert truth
        truth.for_each(|&bit| assert_eq!(reader.read_bit().unwrap(), bit));
        // read past EOF
        (0..16).for_each(|_| assert_eq!(reader.read_bit().unwrap(), 0));
    }

    #[test]
    fn read_u32_complete() {
        let data = b"\xde\xad\xbe\xef";
        let mut reader = ACReader::new(data.as_ref());
        assert_eq!(reader.read_u32().unwrap(), 0xdeadbeef);
        // read past EOF
        (0..16).for_each(|_| assert_eq!(reader.read_bit().unwrap(), 0));
    }

    #[test]
    fn read_u32_incomplete() {
        let data = b"\xde\xad";
        let mut reader = ACReader::new(data.as_ref());
        assert_eq!(reader.read_u32().unwrap(), 0xdead0000);
        // read past EOF
        (0..16).for_each(|_| assert_eq!(reader.read_bit().unwrap(), 0));
    }

    #[test]
    fn write_bits_across_byte_boundary() {
        let mut data = [0; 2];
        let truth = [0b111__0_111_0, 0b000__11111];
        let mut writer = ACWriter::new(data.as_mut());
        (0..3).for_each(|_| writer.write_bit(1).unwrap());
        (0..3).for_each(|_| writer.inc_parity());
        (0..5).for_each(|_| writer.write_bit(0).unwrap());
        writer.flush(u32::MAX).unwrap();
        assert_eq!(data, truth);
    }

    #[test]
    fn write_parity_bits_across_byte_boundary() {
        let mut data = [0; 2];
        let truth = [0b111__0_1111, 0b11_0__11111];
        let mut writer = ACWriter::new(data.as_mut());
        (0..3).for_each(|_| writer.write_bit(1).unwrap());
        (0..6).for_each(|_| writer.inc_parity());
        (0..2).for_each(|_| writer.write_bit(0).unwrap());
        writer.flush(u32::MAX).unwrap();
        assert_eq!(data, truth);
    }

    #[test]
    fn flush_aligned() {
        let truth = [0xff, 0xde];
        let mut data = [0; 2];
        let mut writer = ACWriter::new(data.as_mut());
        (0..8).for_each(|_| writer.write_bit(1).unwrap());
        writer.flush(0xdeadbeef).unwrap();
        assert_eq!(data, truth);
    }

    #[test]
    fn flush_unaligned() {
        let truth = [0xfe, 0x00];
        let mut data = [0; 2];
        let mut writer = ACWriter::new(data.as_mut());
        (0..7).for_each(|_| writer.write_bit(1).unwrap());
        writer.flush(0x00adbeef).unwrap();
        assert_eq!(data, truth);
    }

    #[test]
    fn flush_with_parity() {
        let truth = [0b1111__0_11_0, 0x00];
        let mut data = [0; 2];
        let mut writer = ACWriter::new(data.as_mut());
        (0..4).for_each(|_| writer.write_bit(1).unwrap());
        (0..2).for_each(|_| writer.inc_parity());
        writer.flush(0x00adbeef).unwrap();
        assert_eq!(data, truth);
    }

    #[test]
    fn flush_with_too_much_parity() {
        let truth = [0b1111__0_111, 0b1111111_0];
        let mut data = [0; 2];
        let mut writer = ACWriter::new(data.as_mut());
        (0..4).for_each(|_| writer.write_bit(1).unwrap());
        (0..10).for_each(|_| writer.inc_parity());
        writer.flush(0x00adbeef).unwrap();
        assert_eq!(data, truth);
    }

    #[test]
    fn flush_with_too_much_parity_aligned() {
        let truth = [0b1111111_0, 0b1111111_0];
        let mut data = [0; 2];
        let mut writer = ACWriter::new(data.as_mut());
        (0..7).for_each(|_| writer.write_bit(1).unwrap());
        (0..7).for_each(|_| writer.inc_parity());
        writer.flush(0x00adbeef).unwrap();
        assert_eq!(data, truth);
    }

    #[test]
    fn flush_only() {
        let truth = b"\xde\x00\x00\x00";
        let mut data = [0; 4];
        let mut writer = ACWriter::new(data.as_mut());
        writer.flush(0xdeadbeef).unwrap();
        assert_eq!(&data, truth);
    }
}
