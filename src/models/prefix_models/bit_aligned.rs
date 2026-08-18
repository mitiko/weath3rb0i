use crate::models::{Counter, CtxModel};
use crate::usize;

macro_rules! define {
    ($name:ident, $bits:expr, $mask:expr) => {
        pub struct $name<C: Counter> {
            stats: Vec<C>,
            ctx: usize,
        }

        impl<C: Counter> $name<C> {
            pub fn new() -> Self {
                Self { stats: vec![C::new(); 1 << ($bits + 1)], ctx: 0 }
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
                self.ctx ^= 1 << $bits;
                self.ctx ^= self.ctx & $mask;
                self.ctx |= usize!(hash & $mask);
            }
        }
    };
}

define!(BitAlignedPrefixModel4, 4, 0xF);
define!(BitAlignedPrefixModel7, 7, 0x7F);
define!(BitAlignedPrefixModel8, 8, 0xFF);
define!(BitAlignedPrefixModel12, 12, 0xFFF);
define!(BitAlignedPrefixModel16, 16, 0xFFFF);
