use crate::{u16, u32};
use std::ops::{Add, Div};

use super::P12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct P24(pub(crate) u32); // 0-256 with 24-bit prec

impl P24 {
    pub const MIN: P24 = P24(1);
    pub const MAX: P24 = P24(0);
    pub const ONE: P24 = P24(1 << 24);
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
