pub mod counter;
pub mod order0;
pub mod order0entropy;
pub mod order1;
pub mod stationary;
pub mod ordern;

pub use self::{counter::*, order0::*, order0entropy::*, order1::*, ordern::*};
pub use crate::state_table::*;

pub trait Model {
    fn predict(&self) -> u16;
    fn update(&mut self, bit: u8);
}

pub trait StationaryModel {
    fn predict(&mut self) -> u16;
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
