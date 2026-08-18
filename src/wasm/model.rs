use super::WasmCounter;
use crate::models::*;

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

impl WasmModel {
    pub fn parse(dsl: String) -> Result<WasmModel, String> {
        let name = dsl.split('(').next().unwrap_or("");
        let rem = dsl.split('(').nth(1).unwrap_or("");
        let params = rem.split(')').next().unwrap_or("").to_string();

        let counter = WasmCounter::parse(params)?;

        match name {
            "PM4" => Ok(WasmModel::PM4(PrefixModel4::new(counter))),
            "PM7" => Ok(WasmModel::PM7(PrefixModel7::new(counter))),
            "PM8" => Ok(WasmModel::PM8(PrefixModel8::new(counter))),
            "PM12" => Ok(WasmModel::PM12(PrefixModel12::new(counter))),
            "PM16" => Ok(WasmModel::PM16(PrefixModel16::new(counter))),
            "BytePM4" => Ok(WasmModel::BytePM4(ByteAlignedPrefixModel4::new(counter))),
            "BytePM7" => Ok(WasmModel::BytePM7(ByteAlignedPrefixModel7::new(counter))),
            "BytePM8" => Ok(WasmModel::BytePM8(ByteAlignedPrefixModel8::new(counter))),
            "BytePM12" => Ok(WasmModel::BytePM12(ByteAlignedPrefixModel12::new(counter))),
            "BytePM16" => Ok(WasmModel::BytePM16(ByteAlignedPrefixModel16::new(counter))),
            "BitPM4" => Ok(WasmModel::BitPM4(BitAlignedPrefixModel4::new(counter))),
            "BitPM7" => Ok(WasmModel::BitPM7(BitAlignedPrefixModel7::new(counter))),
            "BitPM8" => Ok(WasmModel::BitPM8(BitAlignedPrefixModel8::new(counter))),
            "BitPM12" => Ok(WasmModel::BitPM12(BitAlignedPrefixModel12::new(counter))),
            "BitPM16" => Ok(WasmModel::BitPM16(BitAlignedPrefixModel16::new(counter))),
            "NibblePM4" => Ok(WasmModel::NibblePM4(NibbleAlignedPrefixModel4::new(
                counter,
            ))),
            "NibblePM7" => Ok(WasmModel::NibblePM7(NibbleAlignedPrefixModel7::new(
                counter,
            ))),
            "NibblePM8" => Ok(WasmModel::NibblePM8(NibbleAlignedPrefixModel8::new(
                counter,
            ))),
            "NibblePM12" => Ok(WasmModel::NibblePM12(NibbleAlignedPrefixModel12::new(
                counter,
            ))),
            "NibblePM16" => Ok(WasmModel::NibblePM16(NibbleAlignedPrefixModel16::new(
                counter,
            ))),
            _ => Err(format!("could not parse model: {name} from '{dsl}'")),
        }
    }
}
