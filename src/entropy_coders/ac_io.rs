use std::io::{Read, Write, self};
use core::fmt::Debug;

use crate::bit_io::{BitReader, BitWriter, ReadError, WriteError};

pub struct ACReader<R> {
    bit_reader: BitReader<R>,
}

impl<R: Read> ACReader<R> {
    pub fn new(inner: R) -> Self {
        Self { bit_reader: BitReader::new(inner) }
    }
    
    /// Read bit or 0 on EOF
    pub fn read_bit(&mut self) -> io::Result<u8> {
        Self::zero(self.bit_reader.read_bit())
    }

    /// Read 4 bytes BE as u32 and pad with 0s if EOF
    pub fn read_u32(&mut self) -> io::Result<u32> {
        // TODO: If we get an EOF or and error, don't continue asking for bytes, assume 0s
        let bytes = [
            Self::zero(self.bit_reader.read_byte())?,
            Self::zero(self.bit_reader.read_byte())?,
            Self::zero(self.bit_reader.read_byte())?,
            Self::zero(self.bit_reader.read_byte())?
        ];
        Ok(u32::from_be_bytes(bytes))
    }

    /// Read 8 bytes BE as u64 and pad with 0s if EOF
    pub fn read_u64(&mut self) -> io::Result<u64> {
        let upper = self.read_u32()?;
        let lower = self.read_u32()?;
        Ok((u64::from(upper) << u32::BITS) | u64::from(lower))
    }

    fn zero(res: Result<u8, ReadError>) -> io::Result<u8> {
        match res {
            Ok(val) => Ok(val),
            Err(ReadError::Eof) => Ok(0),
            Err(ReadError::Other(kind)) => Err(io::Error::from(kind))
        }
    }
}

pub struct ACWriter<W> {
    bit_writer: BitWriter<W>,
    rev_bits: u64
}

impl <W: Write> ACWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { bit_writer: BitWriter::new(inner), rev_bits: 0 }
    }

    pub fn write_bit<T>(&mut self, possib_bit: T) -> io::Result<()>
    where T: TryInto<u8>, T::Error: Debug {
        let bit = possib_bit.try_into().expect("Provided value wasn't a valid bit");
        debug_assert!(bit == 0 || bit == 1, "Provided value wasn't a valid bit");

        self.bit_writer.write(bit)?;
        while self.rev_bits > 0 {
            self.bit_writer.write(bit ^ 1)?;
            self.rev_bits -= 1;
        }

        Ok(())
    }

    pub fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    pub fn flush(&mut self, mut lowx: u32) -> io::Result<()> {
        dbg!(&self.bit_writer.bit_queue);
        dbg!(self.rev_bits);
        dbg!(lowx);

        loop {
            match self.bit_writer.flush() {
                Ok(_) => return Ok(()),
                Err(WriteError::NonemptyBitQueueOnFlush) => {},
                Err(WriteError::Other(kind)) => return Err(io::Error::from(kind))
            };
            self.write_bit(lowx >> (u32::BITS - 1))?;
            lowx <<= 1;
            if self.rev_bits > 0 { continue; }
        }
    }
}
