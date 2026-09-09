use crate::{u16, u32};
use std::{
    num::NonZeroU16,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct P12(u16); // 0-16 with 12-bit prec

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct P24(u32); // 0-256 with 24-bit prec

impl P24 {
    pub const MIN: P24 = P24(1);
    pub const MAX: P24 = P24(0);
    pub const ONE: P24 = P24(1 << 24);
}

impl P12 {
    pub const MIN: P12 = P12(1);
    pub const MAX: P12 = P12(0);
    pub const ONE: P12 = P12(1 << 12);
    pub const TWO: P12 = P12(2 << 12);
    pub const FOUR: P12 = P12(4 << 12);
    pub const HALF: P12 = P12(8 << 12);
}

// treats u16 as 16-bit precision
impl From<u16> for P12 {
    fn from(value: u16) -> Self {
        let x = value >> 3;
        let z = (x >> 1) + (x & 1);
        if z == 0 {
            P12(1)
        } else {
            P12(z)
        }
    }
}

impl From<P12> for f64 {
    fn from(value: P12) -> Self {
        if value == P12::MAX {
            return 16.0;
        }
        f64::from(value.0) / f64::from(1 << 12)
    }
}

impl From<P12> for u32 {
    fn from(value: P12) -> Self {
        if value == P12::MAX {
            return 16 << 12;
        }
        u32::from(value.0)
    }
}

// treats u16 as 16-bit precision
impl Mul<u16> for P12 {
    type Output = P12;

    fn mul(self, rhs: u16) -> Self::Output {
        // 12 bit precision * 16 bit precision = 28 bit precision
        let x = u32::from(self) * u32::from(rhs);
        let x = x >> 15;
        let z = u16!((x >> 1) + (x & 1));
        if z == 0 {
            P12::MIN
        } else {
            P12(z)
        }
    }
}

impl Div for P12 {
    type Output = P12;

    fn div(self, rhs: Self) -> Self::Output {
        let a = u32::from(self) * (1 << 12);
        let b = u32::from(rhs);
        let d = a / b;
        if d == 16 << 12 {
            return P12::MAX;
        }
        P12(u16!(d))
    }
}

impl Sub for P12 {
    type Output = P12;

    fn sub(self, rhs: Self) -> Self::Output {
        let a = u32::from(self);
        let b = u32::from(rhs);
        if a <= b {
            return P12::MIN;
        }
        P12(u16!(a - b))
    }
}

impl Mul for P12 {
    type Output = P24;

    fn mul(self, rhs: Self) -> Self::Output {
        let a = u32::from(self);
        let b = u32::from(rhs);
        P24(a * b)
    }
}

impl From<P12> for P24 {
    fn from(value: P12) -> Self {
        P24(u32::from(value) << 12)
    }
}

impl Add for P24 {
    type Output = P24;

    fn add(self, rhs: Self) -> Self::Output {
        if self == P24::MAX || rhs == P24::MAX {
            return P24::MAX;
        }
        P24(self.0 + rhs.0)
    }
}

impl Add for P12 {
    type Output = P24;

    fn add(self, rhs: Self) -> Self::Output {
        P24::from(self) + P24::from(rhs)
    }
}

impl From<P24> for u64 {
    fn from(value: P24) -> Self {
        if value == P24::MAX {
            return 256 << 12;
        }
        u64::from(value.0)
    }
}

impl Div for P24 {
    type Output = P24;

    fn div(self, rhs: Self) -> Self::Output {
        let a = u64::from(self) * (1 << 24);
        let b = u64::from(rhs);
        let d = a / b;
        if d == 256 << 24 {
            return P24::MAX;
        }
        P24(u32!(d))
    }
}

impl TryFrom<P24> for u16 {
    type Error = &'static str;

    fn try_from(value: P24) -> Result<Self, Self::Error> {
        if value.0 >= P24::ONE.0 {
            return Err("P24 value is too large to convert to u16");
        }
        let x = value.0 >> 7;
        let p = (x >> 1) + (x & 1);
        Ok(u16!(p))
    }
}

// treats u16 as integer
pub fn log2(p: u16) -> P12 {
    if p == 0 {
        panic!("log2 undefined for 0");
    }
    if p == 1 {
        return P12::MIN;
    }
    if p == 65535 {
        return P12::MAX;
    }
    if p == 2 {
        return P12::ONE;
    }
    if p == 3 {
        return P12(6492);
    }
    if p == 4 {
        return P12::TWO;
    }
    if p == 8 {
        return P12(3 << 12);
    }
    if p == 16 {
        return P12::FOUR;
    }
    if p == 32 {
        return P12(5 << 12);
    }
    if p == 64 {
        return P12(6 << 12);
    }
    if p == 128 {
        return P12(7 << 12);
    }
    if p == 256 {
        return P12::HALF;
    }
    if p == 512 {
        return P12(9 << 12);
    }
    if p == 1024 {
        return P12(10 << 12);
    }
    if p == 2048 {
        return P12(11 << 12);
    }
    if p == 4096 {
        return P12(12 << 12);
    }
    if p == 8192 {
        return P12(13 << 12);
    }
    if p == 16384 {
        return P12(14 << 12);
    }
    if p == 32768 {
        return P12(15 << 12);
    }
    return P12(13);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_log2_zero() {
        let result = std::panic::catch_unwind(|| log2(0));
        assert!(result.is_err());
    }

    #[test]
    pub fn test_log2() {
        const EPSILON: f64 = 1.0 / (1 << 12) as f64;

        let mut count = 0;
        for x in 1..=u16::MAX {
            let p = f64::from(log2(x));
            let y = f64::from(x).log2();
            // debug_assert!((y - p).abs() <= EPSILON, "log2({}) = {}, expected {} (diff = {} > {})", x, p, y, (y - p).abs(), EPSILON);
            if (y - p).abs() <= EPSILON {
                count += 1;
            }
        }
        assert_eq!(count, u16::MAX);
    }
}
