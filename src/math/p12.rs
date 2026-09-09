use crate::{u16, u32};
use std::ops::{Add, Div, Mul, Sub};

use super::P24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct P12(pub(crate) u16); // 0-16 with 12-bit prec

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
            P12::MIN
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

impl Mul<u16> for P12 {
    type Output = P12;

    fn mul(self, rhs: u16) -> Self::Output {
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

impl Add for P12 {
    type Output = P24;

    fn add(self, rhs: Self) -> Self::Output {
        P24::from(self) + P24::from(rhs)
    }
}
