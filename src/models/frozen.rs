use crate::{
    models::{AdaptiveModel, Model},
    unroll_for, Analytics,
};

pub struct FrozenModel<T: AdaptiveModel> {
    pub model: T,
}

impl<T: AdaptiveModel> FrozenModel<T> {
    pub fn new(model: T) -> Self {
        Self { model }
    }

    pub fn train(&mut self, data: &[u8]) {
        for byte in data {
            unroll_for!(bit in byte, {
                self.model.adapt(bit);
                self.model.update(bit);
            });
        }
    }
}

impl<T: AdaptiveModel + Clone> Clone for FrozenModel<T> {
    fn clone(&self) -> Self {
        Self { model: self.model.clone() }
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

impl<M: AdaptiveModel + Analytics> Analytics for FrozenModel<M> {
    fn log(&mut self) -> serde_json::Value {
        serde_json::json!({
            "p": self.model.predict(),
            "s": 0, // frozen model does not have state
            "model": self.model.log(),
        })
    }

    fn metadata() -> serde_json::Value {
        serde_json::json!({
            "type": "model/FrozenModel",
            "description": "Adaptive model that is frozen after training. Does not adapt to new data.",
            "children": {
                "model": M::metadata(),
            },
        })
    }
}
