pub struct OpinionMixer2;

const HALF: u16 = 1 << 15;

impl OpinionMixer2 {
    pub fn mix(&self, p1: u16, p2: u16) -> u16 {
        let diff1 = if p1 >= HALF { p1 - HALF } else { HALF - p1 };
        let diff2 = if p2 >= HALF { p2 - HALF } else { HALF - p2 };
        return if diff1 >= diff2 { p1 } else { p2 };
    }
}
