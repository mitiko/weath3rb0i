use crate::math;
use crate::math::P12;
use crate::math::P24;
use crate::mixers::Mixer2;
use crate::u16;

pub struct ConfidenceMixer2;
pub struct MeanMixer2;
pub struct AbsWeightMixer2;
pub struct EntropyWeightMixer2;

const HALF: u16 = 1 << 15;

impl Mixer2 for ConfidenceMixer2 {
    fn mix(&self, p1: u16, p2: u16) -> u16 {
        let d1 = p1.abs_diff(HALF);
        let d2 = p2.abs_diff(HALF);
        return if d1 >= d2 { p1 } else { p2 };
    }

    fn update(&mut self, _bit: u8) {}
}

impl Mixer2 for MeanMixer2 {
    fn mix(&self, p1: u16, p2: u16) -> u16 {
        let x = p1.saturating_add(p2);
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

impl Mixer2 for EntropyWeightMixer2 {
    fn mix(&self, p1: u16, p2: u16) -> u16 {
        // w = 1 / H(P)
        // H(P) = -P*log2(P) - (1-P)*log2(1-P)
        // H(p) = -p/65536*log2(p/65536) - (1-p/65536)*log2(1-p/65536)
        // H(p) = -p/65536*(log2(p)-16) - (1-p/65536)*(log2(65536-p)-16)
        // H(p) = -p/65536*log2(p) + p/65536*16 - (1-p/65536)*log2(65536-p) + (1-p/65536)*16
        // H(p) = 16 - p/65536*log2(p) - (1-p/65536)*log2(65536-p) -> P12
        let h1 = P12::MAX - math::log2(p1) * p1 - math::log2(65535 - p1 + 1) * (65535 - p1 + 1);
        let h2 = P12::MAX - math::log2(p2) * p2 - math::log2(65535 - p2 + 1) * (65535 - p2 + 1);
        let w1 = P12::ONE / h1;
        let w2 = P12::ONE / h2;
        // TODO: P11 with 0-32 range will have 32-bit div instead of 64-bit
        // TODO: P12 / P12 -> P24
        let p = (w1 * p1 + w2 * p2) / (w1 + w2);
        return u16::try_from(p).unwrap();
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
