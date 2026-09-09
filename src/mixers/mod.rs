pub mod static_mixers;

pub trait Mixer2 {
    fn mix(&self, p1: u16, p2: u16) -> u16;
    fn update(&mut self, bit: u8);
}
