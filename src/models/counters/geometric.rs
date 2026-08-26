use super::Counter;
use crate::Analytics;

/// Exponential moving average of the bit, as in fpaq0p.
///
/// Each update moves p by 1/2^RATE of its distance to the bound, so an
/// observation from k bits ago carries weight (1 - 2^-RATE)^k. fpaq0p uses
/// RATE = 5 over a 13-bit probability; this is the same rate at 16 bits.
#[derive(Copy, Clone)]
pub struct GeometricCounter<const RATE: u32> {
    p: u16,
}

impl<const RATE: u32> GeometricCounter<RATE> {
    pub fn new() -> Self {
        Self { p: 1 << 15 }
    }
}

impl<const RATE: u32> Counter for GeometricCounter<RATE> {
    fn p(&self) -> u16 {
        self.p
    }

    fn update(&mut self, bit: u8) {
        self.p = match bit {
            0 => self.p - (self.p >> RATE),
            _ => self.p + ((u16::MAX - self.p) >> RATE),
        };
    }
}

impl<const RATE: u32> Analytics for GeometricCounter<RATE> {
    fn log(&mut self) -> serde_json::Value {
        serde_json::json!({
            "p": self.p,
        })
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "counter/GeometricCounter",
            "description": "16-bit exponential moving average, fpaq0p style",
            "vars": { "p": "u16" },
            // counter specific
            "size": 2,
            "rate": RATE,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const HALF: u16 = 1 << 15;
    const RATE: u32 = 5;

    #[test]
    fn empty() {
        let counter = GeometricCounter::<RATE>::new();
        assert_eq!(counter.p(), HALF);
    }

    /// The up-step uses u16::MAX where fpaq0p would use 1<<16, which doesn't
    /// fit in a u16. That costs one unit of precision, not a clamp.
    #[test]
    fn one_bit_each_way() {
        let mut down = GeometricCounter::<RATE>::new();
        down.update(0);
        assert_eq!(HALF - down.p(), 1024);

        let mut up = GeometricCounter::<RATE>::new();
        up.update(1);
        assert_eq!(up.p() - HALF, 1023);
    }

    /// The step truncates to zero before p reaches either end, so the coder
    /// never sees a probability of 0 or 1 and no clamp is needed. The bounds
    /// sit at 2^RATE - 1 on both sides, which is the counter's confidence
    /// limit -- tightening RATE to adapt faster also makes it less confident.
    #[test]
    fn bounded_without_a_clamp() {
        let mut counter = GeometricCounter::<RATE>::new();
        for _ in 0..10_000 {
            counter.update(0);
        }
        assert_eq!(counter.p(), (1 << RATE) - 1);

        let mut counter = GeometricCounter::<RATE>::new();
        for _ in 0..10_000 {
            counter.update(1);
        }
        assert_eq!(counter.p(), u16::MAX - ((1 << RATE) - 1));
    }

    #[test]
    fn rate_sets_the_bounds() {
        let mut fast = GeometricCounter::<3>::new();
        let mut slow = GeometricCounter::<9>::new();
        for _ in 0..10_000 {
            fast.update(1);
            slow.update(1);
        }
        assert_eq!(fast.p(), u16::MAX - 7);
        assert_eq!(slow.p(), u16::MAX - 511);
    }
}
