use crate::{history::History, models::CtxModel, unroll_for, Analytics};

pub struct CtxModelRunner<H: History, M: CtxModel> {
    history: H,
    model: M,
}

impl<H: History, M: CtxModel> CtxModelRunner<H, M> {
    pub fn new(history: H, model: M) -> Self {
        Self { history, model }
    }

    /// One bit, returning what the model predicted before seeing it.
    pub fn run_bit(&mut self, bit: u8) -> u16 {
        let p = self.model.predict();

        self.model.adapt(bit);
        self.history.update(bit);
        self.model.set_ctx(self.history.hash());

        p
    }

    /// Keeps its state, so consecutive buffers continue one run.
    pub fn run(&mut self, buf: &[u8]) -> Vec<u16> {
        let mut probs = Vec::with_capacity(buf.len() * 8);
        for &byte in buf {
            unroll_for!(bit in byte, {
                probs.push(self.run_bit(bit));
            });
        }
        probs
    }
}

impl<H, M> Analytics for CtxModelRunner<H, M>
where
    H: History + Analytics,
    M: CtxModel + Analytics,
{
    fn log(&mut self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model.log(),
            "history": self.history.log(),
        })
    }

    fn metadata(&self) -> serde_json::Value {
        let model = self.model.metadata();
        let history = self.history.metadata();
        serde_json::json!({
            "type": "runner/CtxModelRunner",
            "description": "Runner for CtxModel with history",
            "children": {
                "model": model,
                "history": history,
            }
        })
    }
}
