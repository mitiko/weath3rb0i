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
        self.model.log()
    }

    fn metadata(&self) -> serde_json::Value {
        let mut value = self.model.metadata();
        if let Some(description) = value["description"].as_str() {
            let new_description =
                format!("Frozen model that does not adapt to new data. {description}");
            value["description"] = serde_json::Value::String(new_description);
        }
        value
    }
}
