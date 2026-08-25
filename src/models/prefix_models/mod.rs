/// Every prefix model is the same shape, so they log the same way: the counter at the
/// current context, plus the lead bit that says how far into the symbol we are.
/// Defined before the modules below, which is what makes it visible inside them.
macro_rules! impl_prefix_analytics {
    ($name:ident, $desc:expr) => {
        impl<C: Counter + crate::Analytics> crate::Analytics for $name<C> {
            fn log(&mut self) -> serde_json::Value {
                let p = self.stats[self.ctx].p();
                serde_json::json!({
                    "counter": self.stats[self.ctx].log(),
                    "lead": self.lead,
                    // model specific: probability & state
                    "p": p,
                    "s": self.ctx,
                })
            }

            fn metadata(&self) -> serde_json::Value {
                serde_json::json!({
                    "type": concat!("model/", stringify!($name)),
                    "description": $desc,
                    "children": { "counter": self.stats[self.ctx].metadata() },
                    "vars": { "lead": "usize" },
                })
            }
        }
    };
}

pub mod prefix_model13;
pub mod prefix_model16;
pub mod prefix_model5;
pub mod prefix_model8;
pub mod prefix_model9;

// nibble trees
pub use self::{prefix_model13::*, prefix_model5::*, prefix_model9::*};

// byte trees
pub use self::{prefix_model16::*, prefix_model8::*};
