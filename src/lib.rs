// TODO: Remove these when all the models are implemented
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod entropy_coding;
pub mod helpers;
pub mod history;
pub mod macros;
pub mod math;
pub mod mixers;
pub mod models;

mod hashmap;
mod state_table;

pub trait Analytics {
    fn log(&mut self) -> serde_json::Value;
    fn metadata(&self) -> serde_json::Value;
}
