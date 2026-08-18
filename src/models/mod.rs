pub mod ac_hash;
pub mod counters;
pub mod ctx_model;
pub mod frozen;
pub mod order0;
pub mod order1;
pub mod ordern;
pub mod ordern_entropy;
pub mod prefix_models;
pub mod runner;

pub use self::{
    counters::*, ctx_model::*, frozen::*, order0::*, order1::*, ordern::*, ordern_entropy::*,
    prefix_models::*, runner::*,
};
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

use crate::mixers::opinion_mixer2::OpinionMixer2;
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
