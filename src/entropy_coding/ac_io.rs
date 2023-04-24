use core::slice::from_mut as into_slice;
use std::io::{self, ErrorKind, Read, Write};

pub trait ACRead {
    fn read_u32(&mut self) -> io::Result<u32>;
    fn read_bit(&mut self) -> io::Result<u8>;
}

pub trait ACWrite {
    fn inc_parity(&mut self);
    fn write_bit(&mut self, bit: u32) -> io::Result<()>;
    fn flush(&mut self, padding: u32) -> io::Result<()>;
}

/// Buffers bits from an `io::Read` instance without changing the bit position.
///
/// For example 0b0101 will produce 0b0000, 0b0100, 0b0000, 0b0001
pub struct ACReader<R> {
    inner: R,
    buf: Option<u8>,
    mask: u8,
}

impl<R: Read> ACReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner, buf: None, mask: 0 }
    }

    // not publicly exposed, helper method
    fn read_byte(&mut self) -> io::Result<u8> {
        debug_assert!(self.buf.is_none());
        let mut byte = 0;
        let result = self.inner.read_exact(into_slice(&mut byte));

        match result {
            Err(err) if err.kind() == ErrorKind::UnexpectedEof => Ok(0),
            _ => result.map(|_| byte),
        }
    }
}

impl<R: Read> ACRead for ACReader<R> {
    /// Read bit or 0 on EOF
    fn read_bit(&mut self) -> io::Result<u8> {
        let byte = if let Some(value) = self.buf {
            self.mask >>= 1;
            value
        } else {
            self.mask = 1 << 7;
            self.read_byte()?
        };
        self.buf = if self.mask == 1 { None } else { Some(byte) };
        Ok((byte & self.mask > 0).into())
    }

    /// Read 4 bytes BE as u32 and pad with 0s if EOF
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
    /// Writes a bit and maintains E3 mapping logic
    fn write_bit(&mut self, bit: u32) -> io::Result<()> {
        let bit = u8::try_from(bit).unwrap_or_default();
        debug_assert!(bit <= 1, "Provided value wasn't a valid bit");

        self.buf = (self.buf << 1) | bit;
        self.idx += 1;
        if self.idx == 8 {
            self.inner.write_all(&[self.buf])?;
            self.idx = 0;
        }

        while self.rev_bits > 0 {
            self.rev_bits -= 1;

            self.buf = (self.buf << 1) | (bit ^ 1);
            self.idx += 1;
            if self.idx == 8 {
                self.inner.write_all(&[self.buf])?;
                self.idx = 0;
            }
        }

        Ok(())
    }

    /// Increases the number of reverse bits to write
    fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    fn flush(&mut self, mut state: u32) -> io::Result<()> {
        // do-while - ensure we write at least one bit
        loop {
            self.write_bit(state >> 31)?;
            state <<= 1;
            if self.idx == 0 {
                break;
            }
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
