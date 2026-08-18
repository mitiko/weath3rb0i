use crate::models::*;
use super::WasmCounter;
    
pub enum WasmModel {
    PM4(PrefixModel4<WasmCounter>),
    PM7(PrefixModel7<WasmCounter>),
    PM8(PrefixModel8<WasmCounter>),
    PM12(PrefixModel12<WasmCounter>),
    PM16(PrefixModel16<WasmCounter>),
    BytePM4(ByteAlignedPrefixModel4<WasmCounter>),
    BytePM7(ByteAlignedPrefixModel7<WasmCounter>),
    BytePM8(ByteAlignedPrefixModel8<WasmCounter>),
    BytePM12(ByteAlignedPrefixModel12<WasmCounter>),
    BytePM16(ByteAlignedPrefixModel16<WasmCounter>),
    BitPM4(BitAlignedPrefixModel4<WasmCounter>),
    BitPM7(BitAlignedPrefixModel7<WasmCounter>),
    BitPM8(BitAlignedPrefixModel8<WasmCounter>),
    BitPM12(BitAlignedPrefixModel12<WasmCounter>),
    BitPM16(BitAlignedPrefixModel16<WasmCounter>),
    NibblePM4(NibbleAlignedPrefixModel4<WasmCounter>),
    NibblePM7(NibbleAlignedPrefixModel7<WasmCounter>),
    NibblePM8(NibbleAlignedPrefixModel8<WasmCounter>),
    NibblePM12(NibbleAlignedPrefixModel12<WasmCounter>),
    NibblePM16(NibbleAlignedPrefixModel16<WasmCounter>),
}

impl CtxModel for WasmModel {
    fn predict(&self) -> u16 {
        match self {
            WasmModel::PM4(m) => m.predict(),
            WasmModel::PM7(m) => m.predict(),
            WasmModel::PM8(m) => m.predict(),
            WasmModel::PM12(m) => m.predict(),
            WasmModel::PM16(m) => m.predict(),
            WasmModel::BytePM4(m) => m.predict(),
            WasmModel::BytePM7(m) => m.predict(),
            WasmModel::BytePM8(m) => m.predict(),
            WasmModel::BytePM12(m) => m.predict(),
            WasmModel::BytePM16(m) => m.predict(),
            WasmModel::BitPM4(m) => m.predict(),
            WasmModel::BitPM7(m) => m.predict(),
            WasmModel::BitPM8(m) => m.predict(),
            WasmModel::BitPM12(m) => m.predict(),
            WasmModel::BitPM16(m) => m.predict(),
            WasmModel::NibblePM4(m) => m.predict(),
            WasmModel::NibblePM7(m) => m.predict(),
            WasmModel::NibblePM8(m) => m.predict(),
            WasmModel::NibblePM12(m) => m.predict(),
            WasmModel::NibblePM16(m) => m.predict(),
        }
    }

    fn set_ctx(&mut self, hash: u32) {
        match self {
            WasmModel::PM4(m) => m.set_ctx(hash),
            WasmModel::PM7(m) => m.set_ctx(hash),
            WasmModel::PM8(m) => m.set_ctx(hash),
            WasmModel::PM12(m) => m.set_ctx(hash),
            WasmModel::PM16(m) => m.set_ctx(hash),
            WasmModel::BytePM4(m) => m.set_ctx(hash),
            WasmModel::BytePM7(m) => m.set_ctx(hash),
            WasmModel::BytePM8(m) => m.set_ctx(hash),
            WasmModel::BytePM12(m) => m.set_ctx(hash),
            WasmModel::BytePM16(m) => m.set_ctx(hash),
            WasmModel::BitPM4(m) => m.set_ctx(hash),
            WasmModel::BitPM7(m) => m.set_ctx(hash),
            WasmModel::BitPM8(m) => m.set_ctx(hash),
            WasmModel::BitPM12(m) => m.set_ctx(hash),
            WasmModel::BitPM16(m) => m.set_ctx(hash),
            WasmModel::NibblePM4(m) => m.set_ctx(hash),
            WasmModel::NibblePM7(m) => m.set_ctx(hash),
            WasmModel::NibblePM8(m) => m.set_ctx(hash),
            WasmModel::NibblePM12(m) => m.set_ctx(hash),
            WasmModel::NibblePM16(m) => m.set_ctx(hash),
        }
    }

    fn adapt(&mut self, bit: u8) {
        match self {
            WasmModel::PM4(m) => m.adapt(bit),
            WasmModel::PM7(m) => m.adapt(bit),
            WasmModel::PM8(m) => m.adapt(bit),
            WasmModel::PM12(m) => m.adapt(bit),
            WasmModel::PM16(m) => m.adapt(bit),
            WasmModel::BytePM4(m) => m.adapt(bit),
            WasmModel::BytePM7(m) => m.adapt(bit),
            WasmModel::BytePM8(m) => m.adapt(bit),
            WasmModel::BytePM12(m) => m.adapt(bit),
            WasmModel::BytePM16(m) => m.adapt(bit),
            WasmModel::BitPM4(m) => m.adapt(bit),
            WasmModel::BitPM7(m) => m.adapt(bit),
            WasmModel::BitPM8(m) => m.adapt(bit),
            WasmModel::BitPM12(m) => m.adapt(bit),
            WasmModel::BitPM16(m) => m.adapt(bit),
            WasmModel::NibblePM4(m) => m.adapt(bit),
            WasmModel::NibblePM7(m) => m.adapt(bit),
            WasmModel::NibblePM8(m) => m.adapt(bit),
            WasmModel::NibblePM12(m) => m.adapt(bit),
            WasmModel::NibblePM16(m) => m.adapt(bit),
        }
    }
}
