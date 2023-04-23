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
        if let Some(val) = self.buf {
            self.mask >>= 1;
            if self.mask == 1 {
                self.buf = None; // last bit
            }
            return Ok((val & self.mask > 0).into());
        }

        self.read_byte().map(|byte| {
            self.buf = Some(byte);
            self.mask = 1 << 7;
            (byte & self.mask > 0).into()
        })
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
    use super::{ACRead, ACReader};

    #[test]
    fn read_bits_test() {
        let data: [u8; 2] = [0b0101_0101, 0b1010_1010];
        let mut reader = ACReader::new(data.as_ref());
        let truth = [0, 1]
            .iter()
            .cycle()
            .take(17)
            .enumerate()
            .filter(|&(i, _)| i != 8)
            .map(|(_, x)| x);

        for &bit in truth {
            assert_eq!(reader.read_bit().unwrap(), bit);
        }
        for _ in 0..16 {
            assert_eq!(reader.read_bit().unwrap(), 0);
        }
    }
}
