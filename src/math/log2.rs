use crate::{u16, u32, usize};
use std::sync::LazyLock;
use super::P12;

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
mod tests {
    use super::*;

    #[test]
    fn test_log2_zero() {
        let result = std::panic::catch_unwind(|| log2(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_log2() {
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
