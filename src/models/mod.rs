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

use crate::{mixers::opinion_mixer2::OpinionMixer2, unroll_for};
pub struct BestOfTwoModel<T, U>
where
    T: Model,
    U: Model,
{
    m1: T,
    m2: U,
    mixer: OpinionMixer2,
}

impl<T, U> BestOfTwoModel<T, U>
where
    T: Model,
    U: Model,
{
    pub fn new(m1: T, m2: U) -> Self {
        Self { m1, m2, mixer: OpinionMixer2 }
    }
}

impl<T, U> Model for BestOfTwoModel<T, U>
where
    T: Model,
    U: Model,
{
    fn predict(&self) -> u16 {
        self.mixer.mix(self.m1.predict(), self.m2.predict())
    }

    fn update(&mut self, bit: u8) {
        self.m1.update(bit);
        self.m2.update(bit);
    }
}
