pub mod opinion_mixer2;

trait Mixer2 {
    fn mix(&self, a: u16, b: u16) -> u16;
    fn update(&mut self, bit: u8);
}
