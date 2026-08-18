pub mod byte_aligned;
pub mod nibble_aligned;
pub mod unaligned;
pub mod bit_aligned;

pub use self::{byte_aligned::*, nibble_aligned::*, unaligned::*, bit_aligned::*};

// Shelwien's
// state = 1<<8 | byte
// for bit in byte: state = state << 1 | bit
// state = ((state << 1 | bit) & mask) | (1 << 8)
// ^ bit updates, byte adaptation
