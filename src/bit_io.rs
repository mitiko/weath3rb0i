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
#![warn(missing_docs)]

use core::slice;
use std::io::{Read, Write, ErrorKind, self};
use self::bit_helpers::BitQueue;
pub use bit_helpers::NibbleRead;

/// The error type for bit read IO operations
#[derive(Debug)]
pub enum ReadError {
    /// End of file error (usually asssume 0s after receiving)
    Eof,
    /// Other `std::io` errors of kind [`std::io::ErrorKind`]
    Other(ErrorKind)
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            ErrorKind::UnexpectedEof => Self::Eof,
            kind => Self::Other(kind)
        }
    }
}

/// The error type for bit write IO operations
#[derive(Debug)]
pub enum WriteError {
    /// Tried to flush the writer but the bit queue wasn't full yet (not 8-bit aligned)
    NonemptyBitQueueOnFlush,
    /// Other `std::io` errors of kind [`std::io::ErrorKind`]
    Other(ErrorKind)
}

/// A BitReader reads bit from an internal `std::io::BufRead` stream
#[derive(Debug)]
pub struct BitReader<R> {
    bit_queue: BitQueue,
    inner: R
}

impl<R: Read> BitReader<R> {
    /// Initializes a BitReader with a stream
    pub fn new(inner: R) -> Self {
        Self { bit_queue: BitQueue::new(), inner }
    }

    /// Reads bit from internal stream or returns `EOF`
    pub fn read_bit(&mut self) -> Result<u8, ReadError> {
        if let Some(bit) = self.bit_queue.try_pop() {
            return Ok(bit);
        }

        let mut byte: u8 = 0;
        self.inner.read_exact(slice::from_mut(&mut byte)).map(|_| {
            self.bit_queue.fill(byte);
            self.bit_queue.try_pop().unwrap()
        })
        .map_err(ReadError::from)
    }

    /// Reads a byte from internal stream or returns EOF
    pub fn read_byte(&mut self) -> Result<u8, ReadError> {
        debug_assert!(self.bit_queue.is_empty()); // not implementing the cold path (as it's not used) for now
        let mut byte: u8 = 0;
        self.inner.read_exact(slice::from_mut(&mut byte))
            .map(|_| byte)
            .map_err(ReadError::from)
    }
}

/// A BitBufWriter writes bits to an internal `std::io::Write` stream
#[derive(Debug)]
pub struct BitWriter<W> {
    inner: W,
    bit_queue: BitQueue
}

impl<W: Write> BitWriter<W> {
    /// Initializes a BitBufWriter with a stream
    pub fn new(inner: W) -> Self {
        Self { inner, bit_queue: BitQueue::new() }
    }

    /// Writes a bit or panics if internal writer doesn't accept more bytes
    pub fn write(&mut self, bit: u8) -> io::Result<()> {
        self.bit_queue.push(bit);
        match self.bit_queue.try_flush() {
            Some(byte) => self.inner.write_all(&[byte]),
            None => Ok(()) // we've pushed the bit to the queue, we've successfully "written" it
        }
    }

    /// Flushes the queue and internal writer
    pub fn flush(&mut self) -> Result<(), WriteError> {
        // It's the caller's responsibility to fill the queue before flushing
        if !self.bit_queue.is_empty() { return Err(WriteError::NonemptyBitQueueOnFlush); }
        if let Some(byte) = self.bit_queue.try_flush() { // cold path, unlikely
            self.inner.write_all(&[byte])
                .map_err(|err| WriteError::Other(err.kind()))?;
        }
        self.inner.flush().map_err(|err| WriteError::Other(err.kind()))
    }
}

mod bit_helpers {
    use std::io::{Read, Bytes};

    /// An 8 element bit queue (with internal store u8)
    /// Handling overflow: panics in debug and discards elements in release
    #[derive(Debug)]
    pub struct BitQueue {
        /// Byte buffer
        t: u8,
        /// Number of bits being held
        count: u8
    }

    impl BitQueue {
        // TODO: Use default?
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
            debug_assert!(!self.is_full()); // looses bits
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
        pub fn try_pop(&mut self) -> Option<u8> {
            if self.is_empty() { return None; }

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
            Some(self.t) // we could zero t, but there's no need to
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
        #[inline(always)]
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
        #[inline(always)]
        pub fn is_empty(&self) -> bool {
            self.count == 0
        }
    }

    /// An `Iterator` over the nibbles of `u8` values of a `Read` instance
    pub struct Nibbles<R> {
        bytes: Bytes<R>,
        nib_buf: Option<u8>
    }

    impl<R: Read> Iterator for Nibbles<R> {
        type Item = u8;

        fn next(&mut self) -> Option<Self::Item> {
            // If we've stored the low nibble, we return it
            if self.nib_buf.is_some() {
                return self.nib_buf.take();
            }
            
            // Otherwise, read a new byte, store the low nibble and return the high nibble
            self.bytes.next().map(|byte_res| {
                let byte = byte_res.unwrap();
                self.nib_buf = Some(byte & 15);
                byte >> 4
            })
        }
    }

    /// The `NibbleRead` trait allows reading nibbles from a [`std::io::Read`] instance
    pub trait NibbleRead<R: Read> {
        /// Transforms this `Read` instance to an `Iterator` over the nibbles of it's `u8` values
        fn nibbles(self) -> Nibbles<R>; 
    }

    impl<R: Read> NibbleRead<R> for R {
        fn nibbles(self) -> Nibbles<R> {
            Nibbles { bytes: self.bytes(), nib_buf: None }
        }
    }
}
