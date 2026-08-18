use crate::models::AdaptiveModel;

// allows reusing a history hash for multiple models
pub trait CtxModel {
    fn predict(&self) -> u16;
    fn set_ctx(&mut self, hash: u32);
    fn adapt(&mut self, bit: u8);
}

// helper struct to convert ctx models into adaptive models
// brings its own u32 history
struct RawModel<T: CtxModel> {
    model: T,
    history: u32,
}

impl<T: CtxModel> AdaptiveModel for RawModel<T> {
    fn predict(&self) -> u16 {
        self.model.predict()
    }

    fn update(&mut self, bit: u8) {
        use crate::history::History;
        self.history.update(bit);
        self.model.set_ctx(self.history.hash());
    }

    fn adapt(&mut self, bit: u8) {
        self.model.adapt(bit);
    }
}
