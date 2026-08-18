use crate::models::{Counter, CtxModel};
use crate::usize;

macro_rules! define {
    ($name:ident, $bits:expr, $mask:expr) => {
        pub struct $name<C: Counter> {
            stats: Vec<C>,
            ctx: usize,
            alignment: usize,
        }

        impl<C: Counter> $name<C> {
            pub fn new() -> Self {
                Self { stats: vec![C::new(); 1 << ($bits + 3)], ctx: 0, alignment: 0 }
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
                self.alignment = (self.alignment + 1) % 8;
                self.ctx = (self.alignment << $bits) | usize!(hash & $mask);
            }
        }
    };
}

define!(ByteAlignedPrefixModel4, 4, 0xF);
define!(ByteAlignedPrefixModel7, 7, 0x7F);
define!(ByteAlignedPrefixModel8, 8, 0xFF);
define!(ByteAlignedPrefixModel12, 12, 0xFFF);
define!(ByteAlignedPrefixModel16, 16, 0xFFFF);
