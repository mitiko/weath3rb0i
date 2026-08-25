use super::WasmModel;
use crate::{history::*, models::*, Analytics};

pub enum WasmHistory {
    Raw(u32),
    AC(ACHistory<FrozenModel<RawModel<WasmModel>>>),
    Huff(HuffHistory),
}

impl History for WasmHistory {
    fn update(&mut self, bit: u8) {
        match self {
            WasmHistory::Raw(h) => h.update(bit),
            WasmHistory::AC(h) => h.update(bit),
            WasmHistory::Huff(h) => h.update(bit),
        }
    }

    fn hash(&mut self) -> u32 {
        match self {
            WasmHistory::Raw(h) => h.hash(),
            WasmHistory::AC(h) => h.hash(),
            WasmHistory::Huff(h) => h.hash(),
        }
    }
}

impl WasmHistory {
    pub fn parse(dsl: String, buf: &[u8]) -> Result<Self, String> {
        // top level split, so a nested model stays one argument
        let (name, args) = super::parse_call(&dsl);
        let params = super::split_args(args);

        match name {
            "Raw" => Ok(WasmHistory::Raw(0)),
            "AC" => {
                let bits: u8 = params
                    .get(0)
                    .ok_or("missing bits parameter for AC history")?
                    .parse()
                    .map_err(|_| "invalid bits parameter for AC history")?;
                let model_def = params
                    .get(1)
                    .ok_or("missing model parameter for AC history")?
                    .to_string();
                let inner = RawModel::new(WasmModel::parse(model_def)?);
                let mut model = FrozenModel::new(inner);
                model.train(buf);
                Ok(WasmHistory::AC(ACHistory::new(bits, model)))
            }
            "Huff" => {
                let huff_size: u8 = params
                    .get(0)
                    .ok_or("missing huff_size parameter for Huff history")?
                    .parse()
                    .map_err(|_| "invalid huff_size parameter for Huff history")?;
                let rem_huff_size: u8 = params
                    .get(1)
                    .ok_or("missing rem_huff_size parameter for Huff history")?
                    .parse()
                    .map_err(|_| "invalid rem_huff_size parameter for Huff history")?;
                Ok(WasmHistory::Huff(HuffHistory::new(
                    buf,
                    huff_size,
                    rem_huff_size,
                )))
            }
            _ => Err(format!("could not parse history: {name} from '{dsl}'")),
        }
    }
}

impl Analytics for WasmHistory {
    fn log(&mut self) -> serde_json::Value {
        match self {
            WasmHistory::Raw(h) => h.log(),
            WasmHistory::AC(h) => h.log(),
            WasmHistory::Huff(h) => h.log(),
        }
    }

    fn metadata(&self) -> serde_json::Value {
        match self {
            WasmHistory::Raw(h) => h.metadata(),
            WasmHistory::AC(h) => h.metadata(),
            WasmHistory::Huff(h) => h.metadata(),
        }
    }
}
