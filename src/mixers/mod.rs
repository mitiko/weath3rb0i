mod interpolated2;

pub use interpolated2::*;

pub trait Mixer {
    fn new() -> Self;
    fn mix(&mut self, p1: u16, p2: u16) -> u16;
    fn update(&mut self, bit: u8);
}
