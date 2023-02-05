use std::io::{self, Read, Write, ErrorKind};
use core::slice::from_mut as into_slice;
use core::fmt::Debug;

pub trait ACRead {
    fn read_u32(&mut self) -> io::Result<u32>;
    fn read_bit(&mut self) -> io::Result<u8>;
}

pub trait ACWrite {
    fn inc_parity(&mut self);
    fn write_bit(&mut self, bit: u32) -> io::Result<()>;
    fn flush(&mut self, padding: u32) -> io::Result<()>;
}

pub trait ZeroedEofExt<T> {
    fn zero_eof(self) -> io::Result<T>;
}

impl<T: Default> ZeroedEofExt<T> for io::Result<T> {
    /// Return result or 0 if EOF
    // T::default() is 0 for u8, u16, u32, u64
    fn zero_eof(self) -> io::Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => match err.kind() {
                ErrorKind::UnexpectedEof => Ok(T::default()),
                _ => Err(err)
            }
        }
    }
}

pub struct ACReader<R> {
    inner: R,
    buf: Option<u8>,
    mask: u8
}

impl<R: Read> ACReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner, buf: None, mask: 0 }
    }

    pub fn read_byte(&mut self) -> io::Result<u8> {
        debug_assert!(self.buf.is_none());
        let mut byte = 0;
        self.inner.read_exact(into_slice(&mut byte))
            .map(|_| byte)
            .zero_eof()
    }
}

impl<R: Read> ACRead for ACReader<R> {
    /// Read bit or 0 on EOF
    fn read_bit(&mut self) -> io::Result<u8> {
        if let Some(val) = self.buf {
            self.mask >>= 1;
            if self.mask == 1 { // last bit
                self.buf = None;
            }
            return Ok((val & self.mask > 0).into());
        }

        self.read_byte().map(|byte| {
            self.buf = Some(byte);
            self.mask = 1 << 7;
            (byte & self.mask > 0).into()
        }).zero_eof()
    }

    /// Read 4 bytes BE as u32 and pad with 0s if EOF
    fn read_u32(&mut self) -> io::Result<u32> {
        let bytes = [
            self.read_byte()?, self.read_byte()?,
            self.read_byte()?, self.read_byte()?
        ];
        Ok(u32::from_be_bytes(bytes))
    }
}

pub struct ACWriter<W> {
    inner: W,
    buf: u8,
    idx: u8,
    rev_bits: u64
}

impl <W: Write> ACWriter<W> {
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
        loop { // ensure we write at least one bit
            self.write_bit(state >> 31)?;
            state <<= 1;
            if self.idx == 0 { break; }
        }

        self.inner.flush()?;
        Ok(())
    }
}
