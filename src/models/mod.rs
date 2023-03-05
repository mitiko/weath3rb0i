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
    mixers: [[InterpolatedMixer2; 15]; 512],
    ctx: u16, nt: NibTree
}

use crate::mixers::Mixer;
use crate::mixers::InterpolatedMixer2;
use self::nib_tree::NibTree;

impl Model for CompoundModel {
    fn new() -> Self {
        Self {
            model_a: Order0::new(),
            model_b: RecordModel::new(),
            mixers: [[InterpolatedMixer2::new(); 15]; 512],
            ctx: 0, nt: NibTree::new()
        }
    }

    fn predict(&mut self) -> u16 {
        let pa = self.model_a.predict();
        let pb = self.model_b.predict();

        let ctx = usize::from(self.ctx);
        let idx = self.nt.get();
        self.mixers[ctx][idx].mix(pa, pb)
    }

    fn update(&mut self, bit: u8) {
        self.model_a.update(bit);
        self.model_b.update(bit);

        const MASK: u16 = (1 << 9) - (1 << 5);
        let ctx = usize::from(self.ctx);
        let idx = self.nt.get();
        self.mixers[ctx][idx].update(bit);

        if let Some(nib) = self.nt.update(bit) {
            let vbit = (self.ctx & 1) ^ 1;
            self.ctx = ((self.ctx << 4) & MASK) | u16::from(nib << 1) | vbit;
        }
    }
}

impl Model4 for CompoundModel {
    fn predict4(&mut self, nib: u8) -> [u16; 4] {
        let pa = self.model_a.predict4(nib);
        let pb = self.model_b.predict4(nib);

        let ctx = usize::from(self.ctx);
        let idxs = self.nt.get4(nib);
        [0, 1, 2, 3].map(|i| self.mixers[ctx][idxs[i]].mix(pa[i], pb[i]))
    }

    fn update4(&mut self, nib: u8) {
        self.model_a.update4(nib);
        self.model_b.update4(nib);

        const MASK: u16 = (1 << 9) - (1 << 5);
        let ctx = usize::from(self.ctx);
        self.nt.get4(nib).into_iter()
            .zip([nib >> 3, (nib >> 2) & 1, (nib >> 1) & 1, nib & 1])
            .for_each(|(idx, bit)| self.mixers[ctx][idx].update(bit));

        let vbit = (self.ctx & 1) ^ 1;
        self.ctx = ((self.ctx << 4) & MASK) | u16::from(nib << 1) | vbit;
    }
}
