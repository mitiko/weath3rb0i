use super::WasmCounter;
use crate::models::*;
use crate::Analytics;

pub enum WasmModel {
    PM5(PrefixModel5<WasmCounter>),
    PM8(PrefixModel8<WasmCounter>),
    PM9(PrefixModel9<WasmCounter>),
    PM13(PrefixModel13<WasmCounter>),
    PM16(PrefixModel16<WasmCounter>),
}

impl CtxModel for WasmModel {
    fn predict(&self) -> u16 {
        match self {
            WasmModel::PM5(m) => m.predict(),
            WasmModel::PM8(m) => m.predict(),
            WasmModel::PM9(m) => m.predict(),
            WasmModel::PM13(m) => m.predict(),
            WasmModel::PM16(m) => m.predict(),
        }
    }

    fn set_ctx(&mut self, hash: u32) {
        match self {
            WasmModel::PM5(m) => m.set_ctx(hash),
            WasmModel::PM8(m) => m.set_ctx(hash),
            WasmModel::PM9(m) => m.set_ctx(hash),
            WasmModel::PM13(m) => m.set_ctx(hash),
            WasmModel::PM16(m) => m.set_ctx(hash),
        }
    }

    fn adapt(&mut self, bit: u8) {
        match self {
            WasmModel::PM5(m) => m.adapt(bit),
            WasmModel::PM8(m) => m.adapt(bit),
            WasmModel::PM9(m) => m.adapt(bit),
            WasmModel::PM13(m) => m.adapt(bit),
            WasmModel::PM16(m) => m.adapt(bit),
        }
    }
}

impl WasmModel {
    pub fn parse(dsl: String) -> Result<WasmModel, String> {
        let (name, args) = super::parse_call(&dsl);
        let params = super::split_args(args);

        let counter_param = params.get(0).ok_or("missing counter parameter for model")?;
        let counter = WasmCounter::parse(counter_param.to_string())?;

        match name {
            "PM5" => Ok(WasmModel::PM5(PrefixModel5::new(counter))),
            "PM8" => Ok(WasmModel::PM8(PrefixModel8::new(counter))),
            "PM9" => Ok(WasmModel::PM9(PrefixModel9::new(counter))),
            "PM13" => Ok(WasmModel::PM13(PrefixModel13::new(counter))),
            "PM16" => Ok(WasmModel::PM16(PrefixModel16::new(counter))),
            _ => Err(format!("could not parse model: {name} from '{dsl}'")),
        }
    }
}

impl Analytics for WasmModel {
    fn log(&mut self) -> serde_json::Value {
        match self {
            WasmModel::PM5(m) => m.log(),
            WasmModel::PM8(m) => m.log(),
            WasmModel::PM9(m) => m.log(),
            WasmModel::PM13(m) => m.log(),
            WasmModel::PM16(m) => m.log(),
        }
    }

    fn metadata(&self) -> serde_json::Value {
        match self {
            WasmModel::PM5(m) => m.metadata(),
            WasmModel::PM8(m) => m.metadata(),
            WasmModel::PM9(m) => m.metadata(),
            WasmModel::PM13(m) => m.metadata(),
            WasmModel::PM16(m) => m.metadata(),
        }
    }
}
