// cfg based on features, which entropy coder to use
// Define AC
// Define rANS?
// tANS, Huffman with static models?

use std::io;

mod ac_io;
pub mod ac32;
pub mod ac48;
pub mod ac64;
#[cfg(feature = "ac32")]
pub use ac32 as arithmetic_coding;
#[cfg(feature = "ac48")]
pub use ac48 as arithmetic_coding;
#[cfg(feature = "ac64")]
pub use ac64 as arithmetic_coding;

pub trait ACEncoder {
    fn encode(&mut self, bit: u8, prob: u16) -> io::Result<()>;
    fn flush(&mut self) -> io::Result<()>;
}

pub trait ACDecoder {
    fn decode(&mut self, prob: u16) -> io::Result<u8>;
}
