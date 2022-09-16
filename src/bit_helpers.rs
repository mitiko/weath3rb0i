/*!
Bit IO helper functions

# Examples

Using a BitReader
```ignore
let reader = BufReader::new(File::open("input")?);
let mut bit_reader = BitReader::new(reader);

let bit = bit_reader.read_bit().unwrap_or(0);
```
// TODO: Add BitReader, BitWriter examples
// TODO: Don't document BitQueue if not public
*/

#![deny(missing_docs)]

use std::{io::{Read, Write}, convert::TryInto};

/// An 8 element bit queue (with internal store u8)
/// Handling overflow: panics in debug and discards elements in release
struct BitQueue {
    /// Byte buffer
    t: u8,
    /// Bits being held
    count: u8
}

impl BitQueue {
    // TODO: Use default
    fn new() -> Self {
        Self { t: 0, count: 0 }
    }

    /// Push a bit in the queue
    /// 
    /// Do not push elements other than 0 and 1!
    /// On overflow (already contains 8 elements):
    /// - in debug mode - panics
    /// - in release mode - discards old elements
    /// 
    /// Example:
    /// ```ignore
    /// let bit_queue = BitQueue::new();
    /// bit_queue.push(1);
    /// bit_queue.push(0);
    /// ```
    fn push(&mut self, bit: u8) {
        debug_assert!(!self.is_full());
        self.t = (self.t << 1) | bit;
        self.count += 1;
    }

    /// Pop a bit from the queue
    /// 
    /// If the queue is empty, returns `None`.
    /// Otherwise `Some(bit)`
    /// 
    /// Example:
    /// ```ignore
    /// let bit_queue: BitQueue::new();
    /// assert_eq!(bit_queue.pop(), None);
    /// bit_queue.push(0);
    /// assert_eq!(bit_queue.pop(), Some(0));
    /// ```
    fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        self.count -= 1;
        Some((self.t >> self.count) & 1)
    }

    /// Tries to flush the queue, only succeeds if full
    /// 
    /// Returns `None` if the queue is not full yet.
    /// Otherwise `Some(byte)`
    /// 
    /// Example:
    /// ```ignore
    /// let bit_queue = BitQueue::new();
    /// assert_eq!(bit_queue.try_flush(), None);
    /// (0..u8::BITS).iter().for_each(|i| bit_queue.push(i & 1));
    /// assert_eq!(bit_queue.try_flush(), Some(0b0101_0101));
    /// ```
    fn try_flush(&mut self) -> Option<u8> {
        if !self.is_full() { return None; }

        self.count = 0;
        Some(self.t)
    }

    /// Fills the bit_queue with a byte
    /// 
    /// On overflow (already not-empty):
    /// - in debug mode: panics
    /// - in release mode: discards old elements
    /// 
    /// Example:
    /// ```ignore
    /// let bit_queue = BitQueue::new();
    /// bit_queue.fill(0x80);
    /// assert_eq!(bit_queue.pop(), Some(1));
    /// assert_eq!(bit_queue.pop(), Some(0));
    /// ```
    fn fill(&mut self, byte: u8) {
        debug_assert!(self.is_empty()); // we shouldn't skip bits
        self.count = u8::BITS.try_into().unwrap();
        self.t = byte;
    }

    /// Checks if the queue is full
    /// 
    /// Example:
    /// ```ignore
    /// let bit_queue = BitQueue::new();
    /// assert_eq!(bit_queue.is_full(), false);
    /// bit_queue.push(0);
    /// assert_eq!(bit_queue.is_full(), false);
    /// bit_queue.pop();
    /// bit_queue.fill(0x80);
    /// assert_eq!(bit_queue.is_full(), true);
    /// ```
    fn is_full(&self) -> bool {
        self.count == u8::BITS.try_into().unwrap()
    }

    /// Checks if the queue is empty
    /// 
    /// Example:
    /// ```ignore
    /// let bit_queue = BitQueue::new();
    /// assert_eq!(bit_queue.is_empty(), true);
    /// bit_queue.push(1);
    /// assert_eq!(bit_queue.is_empty(), false);
    /// ```
    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

// TODO: Examples for BitReader, BitWriter
const DEFAULT_BUFFER_SIZE: usize = 1 << 13; // 8KiB

#[allow(clippy::upper_case_acronyms)]
/// EOF symbol (for error handling)
pub struct EOF;

/// A BitBufReader reads bit from an internal `std::io::Read` stream
pub struct BitBufReader<TRead: Read, const N: usize = DEFAULT_BUFFER_SIZE> {
    stream: TRead,
    buf: [u8; N],
    bit_queue: BitQueue
}

impl<TRead: Read, const N: usize> BitBufReader<TRead, N> {
    /// Initializes a BitBufReader with a stream
    pub fn new(stream: TRead) -> Self {
        Self { stream, buf: [0; N], bit_queue: BitQueue::new() }
    }

    /// Reads bit from internal stream or returns `EOF`
    pub fn read_bit(&mut self) -> Result<u8, EOF> {
        if let Some(bit) = self.bit_queue.pop() {
            return Ok(bit);
        }

        if matches!(self.stream.read(&mut self.buf), Ok(n) if n == 1) { // if-let chains are unstable yet
            self.bit_queue.fill(self.buf[0]);
        }

        self.bit_queue.pop().ok_or(EOF)
    }
}

/// A BitBufWriter writes bits to an internal `std::io::Write` stream
pub struct BitBufWriter<TWrite: Write, const N: usize = DEFAULT_BUFFER_SIZE> {
    stream: TWrite,
    buf: [u8; N],
    idx: usize,
    bit_queue: BitQueue
}

impl<TWrite: Write, const N: usize> BitBufWriter<TWrite, N> {
    /// Initializes a BitBufWriter with a stream
    pub fn new(stream: TWrite) -> Self {
        Self { stream, buf: [0; N], idx: 0, bit_queue: BitQueue::new() }
    }

    /// Writes a bit or panics if internal writer doesn't accept more bytes
    pub fn write_bit(&mut self, bit: u8) {
        println!("WRITING BIT: {}", bit);
        self.bit_queue.push(bit);
        if let Some(byte) = self.bit_queue.try_flush() {
            let bytes_written = self.stream.write(&[byte]).unwrap_or(0);
            assert!(bytes_written != 0, "BitBufWriter failed to write byte");
        }
    }

    /// Pads the remaining bits with a byte and flushes the internal writer
    pub fn flush(&mut self, mut padding_byte: u8) {
        while !self.bit_queue.is_empty() {
            // TODO: Do a single shift?
            self.write_bit((padding_byte >> (u8::BITS - 1)) & 1);
            padding_byte <<= 1;
        }
        debug_assert!(self.bit_queue.is_empty());
        self.stream.flush().unwrap();
    }

    /// Tries to flush the internal writer with no padding byte
    pub fn try_flush(&mut self) {
        assert!(self.bit_queue.is_empty(), "BitQueue wasn't empty");
        self.stream.flush().unwrap();
    }
}
