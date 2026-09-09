pub mod ac_hash;
pub mod counters;
pub mod ctx_model;
pub mod prefix_models;
pub mod runner;

pub use self::{counters::*, ctx_model::*, prefix_models::*, runner::*};
pub use crate::state_table::*;

pub trait Model {
    fn predict(&self) -> u16;
    fn update(&mut self, bit: u8);
}

// stricter interface for adaptive models
// prefer implementing this trait over Model for adaptive models
pub trait AdaptiveModel {
    fn predict(&self) -> u16;
    fn adapt(&mut self, bit: u8);
    fn update(&mut self, bit: u8);
}

// adaptive models are automatically models
impl<T: AdaptiveModel> Model for T {
    fn predict(&self) -> u16 {
        T::predict(self)
    }

    fn update(&mut self, bit: u8) {
        T::adapt(self, bit);
        T::update(self, bit);
    }
}

pub trait SerializableModel {
    fn write(&self) -> Vec<u8>;
    fn read(data: &[u8]) -> Self;
}

pub trait FreezeModel: AdaptiveModel {
    type Frozen: SerializableModel;

    fn train(&mut self, data: &[u8]) {
        for byte in data {
            unroll_for!(bit in byte, {
                self.adapt(bit);
                self.update(bit);
            });
        }
    }
    fn freeze(&self) -> Self::Frozen;
}

use crate::{
    mixers::{static_mixers::ConfidenceMixer2, Mixer2},
    unroll_for,
};
pub struct Composite2<M1, M2, X>
where
    M1: Model,
    M2: Model,
    X: Mixer2,
{
    m1: M1,
    m2: M2,
    mixer: X,
}

impl<M1, M2, X> Composite2<M1, M2, X>
where
    M1: Model,
    M2: Model,
    X: Mixer2,
{
    pub fn new(m1: M1, m2: M2, mixer: X) -> Self {
        Self { m1, m2, mixer }
    }
}

impl<M1, M2, X> Model for Composite2<M1, M2, X>
where
    M1: Model,
    M2: Model,
    X: Mixer2,
{
    fn predict(&self) -> u16 {
        self.mixer.mix(self.m1.predict(), self.m2.predict())
    }

    fn update(&mut self, bit: u8) {
        self.m1.update(bit);
        self.m2.update(bit);
        self.mixer.update(bit);
    }
}
