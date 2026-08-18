use crate::models::*;

#[derive(Copy, Clone)]
pub enum WasmCounter {
    Counter4(Counter4),
}

impl Counter for WasmCounter {
    fn p(&self) -> u16 {
        match self {
            WasmCounter::Counter4(c) => c.p(),
        }
    }

    fn update(&mut self, bit: u8) {
        match self {
            WasmCounter::Counter4(c) => c.update(bit),
        }
    }
}

impl WasmCounter {
    pub fn parse(dsl: String) -> Result<Self, String> {
        let name = dsl.split('(').next().unwrap_or("");

        match name {
            "Counter4" => Ok(WasmCounter::Counter4(Counter4::new())),
            _ => Err(format!("could not parse counter: {name} from '{dsl}'")),
        }
    }
}
