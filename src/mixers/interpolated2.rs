use super::Mixer;

const ONE: i64 = 1 << 16;
const ALPHA: i64 = {
    let lr: f64 = 0.2; // learning rate in [0, 1]
    (lr * 256.0 / std::f64::consts::LN_2) as i64
};

#[derive(Clone, Copy)]
pub struct InterpolatedMixer2 {
    w: i64,
    p: i64,
    dp: i64,
}

impl Mixer for InterpolatedMixer2 {
    fn new() -> Self { Self { w: 1 << 15, dp: 0, p: 1 } }

    fn mix(&mut self, p1: u16, p2: u16) -> u16 {
        debug_assert!(self.w >= 0 && self.w <= 1 << 16);
        self.dp = i64::from(p1) - i64::from(p2);
        self.p = {
            let p = (self.w * self.dp + (i64::from(p2) << 16)) >> 15;
            if p == 0 { 1 } else { (p >> 1) + (p & 1) }
        };
        u16::try_from(self.p).unwrap()
    }

    fn update(&mut self, bit: u8) {
        let y = i64::from(bit) << 16;
        let num = self.dp * (self.p - y) * ALPHA * (1 << 9);
        let den = self.p * (ONE - self.p);
        let dw = { let f = num / den; (f >> 1) + (f & 1) }; // round
        self.w = {
            let wp = self.w - dw;
            if wp < 0 { 0 } else if wp > ONE { ONE } else { wp }
        };
    }
}
