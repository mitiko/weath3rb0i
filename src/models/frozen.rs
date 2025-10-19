use crate::models::{AdaptiveModel, Model};

pub struct FrozenModel<T: AdaptiveModel> {
    pub model: T,
}

impl<T: AdaptiveModel> FrozenModel<T> {
    pub fn new(model: T) -> Self {
        Self { model }
    }
}

impl<T: AdaptiveModel> Model for FrozenModel<T> {
    fn predict(&self) -> u16 {
        self.model.predict()
    }

    fn update(&mut self, bit: u8) {
        self.model.update(bit);
    }
}
