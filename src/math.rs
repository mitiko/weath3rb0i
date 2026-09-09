use crate::{u16, u32, usize};
use std::{
    num::NonZeroU16,
    ops::{Add, Div, Mul, Shl, Sub},
    sync::LazyLock,
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
        if d >= 16 << 12 {
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

// log2(1 + index / 4096), in P12 fixed-point representation.
// The leading (whole) part of the logarithm is calculated directly, so only
// the normalized fractional part needs to be stored.
static LOG2_TABLE: LazyLock<[u16; 4096]> = LazyLock::new(build_log2_table);

/// Builds the normalized log2 lookup table directly at P12 precision.
pub fn build_log2_table() -> [u16; 4096] {
    let scale = f64::from(1 << 12);
    std::array::from_fn(|index| {
        let normalized = 1.0 + f64::from(index as u32) / scale;
        (normalized.log2() * scale).round() as u16
    })
}

// treats u16 as integer
pub fn log2(p: u16) -> P12 {
    if p == 0 {
        panic!("log2 undefined for 0");
    }
    if p == 1 {
        return P12::MIN;
    }

    // Let base = 2^whole. Then p = base + n, where 0 <= n < base:
    // log2(p) = floor(log2(p)) + log2(1 + n / base)
    let whole = u16!(15 - p.leading_zeros());
    let n = u32::from(p - (1 << whole));
    let mask = (1 << whole) - 1;

    // index = floor(n / base * 4096)
    let index = (n << 12) >> whole;
    let rem = (n << 12) & mask;

    // no lookup when close to 2.0
    if index == 4096 || index == 4095 {
        return P12((whole + 1) << 12);
    }

    // interpolate
    let a = u32::from(LOG2_TABLE[usize!(index)]);
    let b = u32::from(LOG2_TABLE[usize!(index + 1)]);
    // f = a + (b-a) * remainder / base
    let x = ((b - a) * rem) >> (whole - 1);
    let f = a + (x >> 1) + (x & 1);
    P12((whole << 12) + u16!(f))
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
            if (y - p).abs() <= EPSILON {
                count += 1;
            }
        }
        assert_eq!(count, u16::MAX as usize);
    }
}
