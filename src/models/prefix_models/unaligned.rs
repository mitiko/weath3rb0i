use crate::models::{Counter, CtxModel};
use crate::usize;

macro_rules! define {
    ($name:ident, $bits:expr, $mask:expr) => {
        pub struct $name<C: Counter> {
            stats: Vec<C>,
            ctx: usize,
        }

        impl<C: Counter> $name<C> {
            pub fn new(counter: C) -> Self {
                Self { stats: vec![counter; 1 << $bits], ctx: 0 }
            }
        }

        impl<C: Counter> CtxModel for $name<C> {
            fn predict(&self) -> u16 {
                self.stats[self.ctx].p()
            }

            fn adapt(&mut self, bit: u8) {
                self.stats[self.ctx].update(bit);
            }

            fn set_ctx(&mut self, hash: u32) {
                self.ctx = usize!(hash & $mask);
            }
        }
    };
}

define!(PrefixModel4, 4, 0xF);
define!(PrefixModel7, 7, 0x7F);
define!(PrefixModel8, 8, 0xFF);
define!(PrefixModel12, 12, 0xFFF);
define!(PrefixModel16, 16, 0xFFFF);
