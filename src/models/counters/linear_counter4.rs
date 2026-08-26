use super::Counter;
use crate::{Analytics, u16};

#[derive(Copy, Clone)]
pub struct Counter4 {
    data: [u16; 2],
}

impl Counter4 {
    pub fn new() -> Self {
        Self { data: [0; 2] }
    }
}

impl Counter for Counter4 {
    fn p(&self) -> u16 {
        let c0 = u64::from(self.data[0]);
        let c1 = u64::from(self.data[1]);
        let p = (1 << 17) * (c1 + 1) / (c0 + c1 + 2);
        u16!((p >> 1) + (p & 1)) // rounding
    }

    fn update(&mut self, bit: u8) {
        if self.data[usize::from(bit)] == u16::MAX {
            self.data[0] = (self.data[0] >> 1) + (self.data[0] & 1);
            self.data[1] = (self.data[1] >> 1) + (self.data[1] & 1);
        }
        self.data[usize::from(bit)] += 1;
    }
}

impl Analytics for Counter4 {
    fn log(&mut self) -> serde_json::Value {
        serde_json::json!({
            "data": self.data,
            // counter specific
            "p": self.p(),
        })
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "counter/Counter4",
            "description": "16-bit counter with rounding and renormalization",
            "vars": { "data": "[u16; 2]" },
            // counter specific
            "size": 4,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const HALF: u16 = 1 << 15;

    #[test]
    fn empty() {
        let counter = Counter4::new();
        assert_eq!(counter.p(), HALF);
    }

    #[test]
    fn zero() {
        let mut counter = Counter4::new();
        counter.update(0);
        assert!(counter.p() < HALF);
        counter.update(1);
        assert_eq!(counter.p(), HALF);
    }

    #[test]
    fn one() {
        let mut counter = Counter4::new();
        counter.update(1);
        assert!(counter.p() > HALF);
        counter.update(0);
        assert_eq!(counter.p(), HALF);
    }

    #[test]
    fn only_zeros() {
        let mut counter = Counter4::new();
        for _ in 0..u16::MAX {
            counter.update(0);
        }
        assert_eq!(counter.p(), 1);
        counter.update(0);
        assert!(counter.p() > 1); // quirky
    }

    #[test]
    fn only_ones() {
        let mut counter = Counter4::new();
        for _ in 0..u16::MAX {
            counter.update(1);
        }
        assert_eq!(counter.p(), u16::MAX);
        counter.update(1);
        assert!(counter.p() < u16::MAX); // quirky
    }

    #[test]
    fn renorm() {
        let mut counter = Counter4::new();
        for _ in 0..u16::MAX {
            counter.update(0);
        }
        for _ in 0..u16::MAX {
            counter.update(1);
        }
        assert_eq!(counter.p(), HALF);
        counter.update(0);
        counter.update(1);
        assert_eq!(counter.p(), HALF);
    }
}
