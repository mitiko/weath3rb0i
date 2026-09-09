use crate::mixers::Mixer2;
use crate::u16;

pub struct ConfidenceMixer2;
pub struct MeanMixer2;
pub struct AbsWeightMixer2;

const HALF: u16 = 1 << 15;

impl Mixer2 for ConfidenceMixer2 {
    fn mix(&self, p1: u16, p2: u16) -> u16 {
        let diff1 = if p1 >= HALF { p1 - HALF } else { HALF - p1 };
        let diff2 = if p2 >= HALF { p2 - HALF } else { HALF - p2 };
        return if diff1 >= diff2 { p1 } else { p2 };
    }

    fn update(&mut self, _bit: u8) {}
}

impl Mixer2 for MeanMixer2 {
    fn mix(&self, p1: u16, p2: u16) -> u16 {
        let x = p1 + p2;
        (x / 2) + (x & 1)
    }

    fn update(&mut self, _bit: u8) {}
}

impl Mixer2 for AbsWeightMixer2 {
    fn mix(&self, p1: u16, p2: u16) -> u16 {
        let w1 = u32::from(p1.abs_diff(HALF));
        let w2 = u32::from(p2.abs_diff(HALF));
        if w1 == 0 && w2 == 0 {
            return HALF;
        }
        let sum = u32::from(p1) * w1 + u32::from(p2) * w2;
        u16!(sum / (w1 + w2))
    }

    fn update(&mut self, _bit: u8) {}
}

mod tests {
    use super::*;

    #[test]
    fn test_confidence_mixer() {
        let mixer = ConfidenceMixer2;
        assert_eq!(mixer.mix(0, 65535), 0);
        assert_eq!(mixer.mix(32768, 32767), 32767);
        assert_eq!(mixer.mix(10000, 20000), 10000);
    }

    #[test]
    fn test_mean_mixer() {
        let mixer = MeanMixer2;
        assert_eq!(mixer.mix(0, 65535), 32768);
        assert_eq!(mixer.mix(32768, 32767), 32768);
        assert_eq!(mixer.mix(10000, 20000), 15000);
    }

    #[test]
    fn test_abs_weight_mixer() {
        let mixer = AbsWeightMixer2;
        assert_eq!(mixer.mix(0, 65535), 32767);
        assert_eq!(mixer.mix(32768, 32768), 32768);
        assert_eq!(mixer.mix(10000, 20000), 13592);
    }
}
