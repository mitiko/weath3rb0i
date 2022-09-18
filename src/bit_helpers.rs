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

use std::io::{Read, Write};
use self::buffers::{BitQueue, DEFAULT_BUFFER_SIZE, ReadBuf, Nibbles};

/// EOF symbol (for error handling)
#[allow(clippy::upper_case_acronyms)]
pub struct EOF;

/// A BitBufReader reads bit from an internal `std::io::Read` stream
pub struct BitBufReader<R, const N: usize = DEFAULT_BUFFER_SIZE> {
    bit_queue: BitQueue,
    byte_queue: ReadBuf<R>
}

impl<R: Read, const N: usize> BitBufReader<R, N> {
    /// Initializes a BitBufReader with a stream
    pub fn new(inner: R) -> Self {
        Self { bit_queue: BitQueue::new(), byte_queue: ReadBuf::new(inner) }
    }

    /// Reads bit from internal stream or returns `EOF`
    pub fn read_bit(&mut self) -> Result<u8, EOF> {
        if let Some(bit) = self.bit_queue.pop() { return Ok(bit); }

        if let Some(byte) = self.byte_queue.pop() {
            self.bit_queue.fill(byte);
            return self.bit_queue.pop().ok_or(EOF);
        }

        self.byte_queue.try_fill();

        if let Some(byte) = self.byte_queue.pop() {
            self.bit_queue.fill(byte);
        }

        self.bit_queue.pop().ok_or(EOF)
    }

    /// Transforms this BitBufReader instance to an Iterator over its nibbles.
    pub fn nibbles(self) -> Nibbles<R> {
        Nibbles::new(self.byte_queue)
    }
}

/// A BitBufWriter writes bits to an internal `std::io::Write` stream
pub struct BitBufWriter<W, const N: usize = DEFAULT_BUFFER_SIZE> {
    stream: W,
    // buf: [u8; N],
    // idx: usize,
    bit_queue: BitQueue
}

impl<W: Write, const N: usize> BitBufWriter<W, N> {
    /// Initializes a BitBufWriter with a stream
    pub fn new(stream: W) -> Self {
        // Self { stream, buf: [0; N], idx: 0, bit_queue: BitQueue::new() }
        Self { stream, bit_queue: BitQueue::new() }
    }

    /// Writes a bit or panics if internal writer doesn't accept more bytes
    pub fn write_bit(&mut self, bit: u8) {
        self.bit_queue.push(bit);
        if let Some(byte) = self.bit_queue.try_flush() {
            self.stream.write_all(&[byte]).expect("BitBufWriter failed to write byte");
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
        self.stream.flush().expect("BitBufWriter.inner couldn't flush");
    }

    /// Tries to flush the internal writer with no padding byte
    pub fn try_flush(&mut self) {
        assert!(self.bit_queue.is_empty(), "BitQueue wasn't empty");
        self.stream.flush().expect("BitBufWriter.inner couldn't flush");
    }
}

mod buffers {
    use std::io::{Read, ErrorKind, Write};

    // TODO: Examples for BitReader, BitWriter
    pub const DEFAULT_BUFFER_SIZE: usize = 1 << 13; // 8KiB

    /// An 8 element bit queue (with internal store u8)
    /// Handling overflow: panics in debug and discards elements in release
    pub struct BitQueue {
        /// Byte buffer
        t: u8,
        /// Bits being held
        count: u8
    }

    impl BitQueue {
        // TODO: Use default
        /// Creates a new empty bit queue
        pub fn new() -> Self {
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
        pub fn push(&mut self, bit: u8) {
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
        pub fn pop(&mut self) -> Option<u8> {
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
        pub fn try_flush(&mut self) -> Option<u8> {
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
        pub fn fill(&mut self, byte: u8) {
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
        pub fn is_empty(&self) -> bool {
            self.count == 0
        }
    }

    pub struct ReadBuf<T, const N: usize = DEFAULT_BUFFER_SIZE> {
        data: [u8; N],
        inner: T,
        consumed: usize,
        filled: usize,
        accepts_more: bool
    }

    impl<T, const N: usize> ReadBuf<T, N> {
        pub fn new(inner: T) -> Self {
            Self { data: [0; N], inner, consumed: 0, filled: 0, accepts_more: true }
        }
    }

    impl<W: Write, const N: usize> ReadBuf<W, N> {
    }

    impl<R: Read, const N: usize> ReadBuf<R, N> {
        pub fn try_fill(&mut self) {
            if !self.accepts_more { return; }

            let mut data = &mut self.data[self.filled..];
            while !data.is_empty() {
                match self.inner.read(data) {
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {},
                    Ok(0) | Err(_) => {
                        self.accepts_more = false;
                        break
                    },
                    Ok(n) => {
                        self.filled += n;
                        data = &mut data[n..];
                    }
                }
            }
        }

        pub fn pop(&mut self) -> Option<u8> {
            if self.consumed == self.filled {
                self.consumed = 0; self.filled = 0;
                return None;
            }

            let byte = self.data[self.consumed];
            self.consumed += 1;
            Some(byte)
        }
    }

    /// An iterator over the nibbles of u8 values of a Read instance
    pub struct Nibbles<R> {
        inner: ReadBuf<R>,
        nib_buf: Option<u8>
    }

    impl<R: Read> Nibbles<R> {
        pub fn new(inner: ReadBuf<R>) -> Self {
            Self { inner, nib_buf: None }
        }
    }

    impl<R: Read> Iterator for Nibbles<R> {
        type Item = u8;

        fn next(&mut self) -> Option<Self::Item> {        
            if let Some(byte) = self.nib_buf.take() {
                return Some(byte & 15);
            }

            if let Some(byte) = self.inner.pop() {
                self.nib_buf = Some(byte);
                return Some(byte >> 4)
            }

            self.inner.try_fill();
            self.inner.pop()
                .map(|byte| { self.nib_buf = Some(byte); byte >> 4 })
        }
    }
}
