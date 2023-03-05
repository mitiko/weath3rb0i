mod order0;
mod record_model;
mod nib_tree;

pub use {order0::*, record_model::*};

pub trait Model {
    fn new() -> Self;
    fn predict(&mut self) -> u16;
    fn update(&mut self, bit: u8);
}

pub trait Model4 : Model {
    fn predict4(&mut self, nib: u8) -> [u16; 4];
    fn update4(&mut self, nib: u8);
}

pub struct CompoundModel {
    model_a: Order0,
    model_b: RecordModel,
    mixer: InterpolatedMixer2
}

use crate::mixers::Mixer;
use crate::mixers::InterpolatedMixer2;
impl Model for CompoundModel {
    fn new() -> Self {
        Self {
            model_a: Order0::new(),
            model_b: RecordModel::new(),
            mixer: InterpolatedMixer2::new()
        }
    }

    fn predict(&mut self) -> u16 {
        let pa = self.model_a.predict();
        let pb = self.model_b.predict();
        self.mixer.mix(pa, pb)
    }

    fn update(&mut self, bit: u8) {
        self.mixer.update(bit);
        self.model_a.update(bit);
        self.model_b.update(bit);
    }
}

impl Model4 for CompoundModel {
    fn predict4(&mut self, nib: u8) -> [u16; 4] {
        let pa = self.model_a.predict4(nib);
        let pb = self.model_b.predict4(nib);
        let mut res = [0; 4];
        res[0] = self.mixer.mix(pa[0], pb[0]);
        self.mixer.update(nib >> 3);
        res[1] = self.mixer.mix(pa[1], pb[1]);
        self.mixer.update((nib >> 2) & 1);
        res[2] = self.mixer.mix(pa[2], pb[2]);
        self.mixer.update((nib >> 1) & 1);
        res[3] = self.mixer.mix(pa[3], pb[3]);
        self.mixer.update(nib & 1);
        res
    }

    fn update4(&mut self, nib: u8) {
        self.model_a.update4(nib);
        self.model_b.update4(nib);
    }
}
