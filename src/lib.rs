// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

// TODO: Remove these when all the models are implemented
#![allow(dead_code)]
#![allow(unused_imports)]

// TODO: #![deny(missing_docs)] on all submodules

pub mod arithmetic_coder;
pub mod bit_io;
pub mod models;

mod hashmap;
mod mixer;
mod smart_context;
mod state_table;

#[macro_use]
pub extern crate debug_unreachable;


