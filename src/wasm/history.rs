use super::WasmModel;
use crate::{history::*, models::RawModel};

pub enum WasmHistory {
    Raw(u32),
    AC(ACHistory<RawModel<WasmModel>>),
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
