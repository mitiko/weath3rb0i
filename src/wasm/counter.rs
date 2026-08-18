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
