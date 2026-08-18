use crate::{history::History, models::CtxModel, unroll_for, Analytics};

pub struct CtxModelRunner<H: History, M: CtxModel> {
    history: H,
    model: M,
}

impl<H: History, M: CtxModel> CtxModelRunner<H, M> {
    pub fn run(&mut self, buf: &[u8]) -> Vec<u16> {
        let mut probs = Vec::with_capacity(buf.len() * 8);
        for &byte in buf {
            unroll_for!(bit in byte, {
                let p = self.model.predict();

                self.model.adapt(bit);
                self.history.update(bit);
                self.model.set_ctx(self.history.hash());

                probs.push(p);
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

    fn metadata() -> serde_json::Value {
        let model = M::metadata();
        let history = H::metadata();
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
