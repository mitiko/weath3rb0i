#[macro_export]
macro_rules! unroll_for {
    ($b:ident in $byte: expr, $x: block) => {
        let mut $b = $byte >> 7;
        $x;
        $b = ($byte >> 6) & 1;
        $x;
        $b = ($byte >> 5) & 1;
        $x;
        $b = ($byte >> 4) & 1;
        $x;
        $b = ($byte >> 3) & 1;
        $x;
        $b = ($byte >> 2) & 1;
        $x;
        $b = ($byte >> 1) & 1;
        $x;
        $b = $byte & 1;
        $x;
    };
}

#[macro_export]
macro_rules! unroll_collect {
    ($bit:ident into $byte:ident, $x: block) => {
        let mut $byte = 0;
        let mut $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
    };
}

#[macro_export]
macro_rules! u8 {
    ($a:expr) => {
        if cfg!(feature = "unsafe_conversions") {
            unsafe { u8::try_from($a).unwrap_unchecked() }
        } else {
            u8::try_from($a).unwrap()
        }
    };
}

#[macro_export]
macro_rules! u16 {
    ($a:expr) => {
        if cfg!(feature = "unsafe_conversions") {
            unsafe { u16::try_from($a).unwrap_unchecked() }
        } else {
            u16::try_from($a).unwrap()
        }
    };
}

#[macro_export]
macro_rules! u32 {
    ($a:expr) => {
        if cfg!(feature = "unsafe_conversions") {
            unsafe { u32::try_from($a).unwrap_unchecked() }
        } else {
            u32::try_from($a).unwrap()
        }
    };
}

#[macro_export]
macro_rules! u64 {
    ($a:expr) => {
        if cfg!(feature = "unsafe_conversions") {
            unsafe { u64::try_from($a).unwrap_unchecked() }
        } else {
            u64::try_from($a).unwrap()
        }
    };
}

#[macro_export]
macro_rules! usize {
    ($a:expr) => {
        if cfg!(feature = "unsafe_conversions") {
            unsafe { usize::try_from($a).unwrap_unchecked() }
        } else {
            usize::try_from($a).unwrap()
        }
    };
}
