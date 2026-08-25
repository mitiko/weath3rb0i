use crate::{
    history::{ACHistory, History, HuffHistory},
    models::*,
};

pub mod counter;
pub mod history;
pub mod model;

pub use self::{counter::*, history::*, model::*};

/// `"Name(a, b(c))"` into `("Name", "a, b(c)")`. No parens means no args.
pub fn parse_call(dsl: &str) -> (&str, &str) {
    let dsl = dsl.trim();
    match dsl.find('(') {
        // only the matching paren, or a nested call loses its own
        Some(i) => {
            let args = dsl[i + 1..].trim_end();
            (dsl[..i].trim(), args.strip_suffix(')').unwrap_or(args))
        }
        None => (dsl, ""),
    }
}

/// Split on commas at the top level only, so a nested call stays one argument.
pub fn split_args(args: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0usize);
    for (i, c) in args.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(args[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = args[start..].trim();
    if !last.is_empty() || !out.is_empty() {
        out.push(last);
    }
    out
}

/// What JS talks to. The runner keeps its state, so consecutive slices continue one run,
/// and the probabilities come back as a Uint16Array ready to colour text with.
#[cfg(target_arch = "wasm32")]
mod bindings {
    use super::{WasmHistory, WasmModel};
    use crate::models::CtxModelRunner;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct Runner {
        inner: CtxModelRunner<WasmHistory, WasmModel>,
    }

    #[wasm_bindgen]
    impl Runner {
        /// `buf` is the whole source, which a trained history needs before the first bit.
        #[wasm_bindgen(constructor)]
        pub fn new(model: &str, history: &str, buf: &[u8]) -> Result<Runner, String> {
            let model = WasmModel::parse(model.to_string())?;
            let history = WasmHistory::parse(history.to_string(), buf)?;
            Ok(Self { inner: CtxModelRunner::new(history, model) })
        }

        pub fn run(&mut self, buf: &[u8]) -> Vec<u16> {
            self.inner.run(buf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_call() {
        assert_eq!(parse_call("Raw"), ("Raw", ""));
        assert_eq!(parse_call("PM8(Counter4)"), ("PM8", "Counter4"));
        assert_eq!(parse_call("AC(12, PM8(Counter4))"), ("AC", "12, PM8(Counter4)"));
    }

    #[test]
    fn nested_args_stay_whole() {
        assert_eq!(split_args("12, PM8(Counter4)"), vec!["12", "PM8(Counter4)"]);
        assert_eq!(split_args("11, 11"), vec!["11", "11"]);
        assert_eq!(split_args(""), Vec::<&str>::new());
    }

    #[test]
    fn parses_the_pieces() {
        assert!(WasmCounter::parse("Counter4".into()).is_ok());
        assert!(WasmModel::parse("PM8(Counter4)".into()).is_ok());
        assert!(WasmModel::parse("NibblePM16(Counter4)".into()).is_ok());
        assert!(WasmHistory::parse("Raw".into(), b"data").is_ok());
        // the nested model has to survive being an argument
        assert!(WasmHistory::parse("AC(12, PM8(Counter4))".into(), b"data").is_ok());
    }

    #[test]
    fn bad_names_are_errors_not_panics() {
        assert!(WasmCounter::parse("Nope".into()).is_err());
        assert!(WasmModel::parse("PM9(Counter4)".into()).is_err());
        assert!(WasmModel::parse("PM8(Nope)".into()).is_err());
        assert!(WasmHistory::parse("Nope".into(), b"data").is_err());
        assert!(WasmHistory::parse("AC(notanumber, PM8(Counter4))".into(), b"data").is_err());
        assert!(WasmHistory::parse("AC(12)".into(), b"data").is_err());
    }
}
