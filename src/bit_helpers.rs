use std::{io::{Read, Write}, convert::TryInto};

struct BitQueue {
    t: u8,    // byte buffer
    count: u8 // bits being held
}

impl BitQueue {
    fn new() -> Self {
        Self { t: 0, count: 0 }
    }

    fn push(&mut self, bit: u8) {
        debug_assert!(!self.is_full());
        self.t = (self.t << 1) | bit;
        self.count += 1;
    }

    fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        self.count -= 1;
        Some((self.t >> self.count) & 1)
    }

    fn try_flush(&mut self) -> Option<u8> {
        if !self.is_full() { return None; }

        self.count = 0;
        Some(self.t)
    }

    fn fill(&mut self, byte: u8) {
        debug_assert!(self.is_empty()); // we shouldn't skip bits
        self.count = u8::BITS.try_into().unwrap();
        self.t = byte;
    }

    fn is_full(&self) -> bool {
        self.count == u8::BITS.try_into().unwrap()
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[allow(clippy::upper_case_acronyms)]
pub struct EOF;

pub struct BitReader<TRead: Read> {
    stream: TRead,
    buf: [u8; 1],
    bit_queue: BitQueue
}

impl<TRead: Read> BitReader<TRead> {
    pub fn new(stream: TRead) -> Self {
        Self { stream, buf: [0; 1], bit_queue: BitQueue::new() }
    }

    pub fn read_bit(&mut self) -> Result<u8, EOF> {
        if let Some(bit) = self.bit_queue.pop() {
            return Ok(bit);
        }

        if matches!(self.stream.read(&mut self.buf), Ok(n) if n == 1) { // if-let chains are unstable yet
            self.bit_queue.fill(self.buf[0]);
        }

        // TODO: Check if this is inlined well
        self.bit_queue.pop().ok_or(EOF)
    }
}

pub struct BitWriter<TWrite: Write> {
    stream: TWrite,
    bit_queue: BitQueue
}

impl<TWrite: Write> BitWriter<TWrite> {
    pub fn new(stream: TWrite) -> Self {
        Self { stream, bit_queue: BitQueue::new() }
    }

    pub fn write_bit(&mut self, bit: u8) {
        self.bit_queue.push(bit);
        if let Some(byte) = self.bit_queue.try_flush() {
            let bytes_written = self.stream.write(&[byte]).unwrap_or(0);
            assert!(bytes_written != 0, "BitWriter failed to write byte");
        }
    }

    pub fn flush(&mut self, mut padding_byte: u8) {
        while !self.bit_queue.is_empty() {
            // println!("Unrool");
            self.write_bit((padding_byte >> (u8::BITS - 1)) & 1);
            padding_byte <<= 1;
        }
        debug_assert!(self.bit_queue.is_empty());
        self.stream.flush().unwrap();
    }

    pub fn try_flush(&mut self) {
        assert!(self.bit_queue.is_empty(), "BitQueue wasn't empty");
        self.stream.flush().unwrap();
    }
}
