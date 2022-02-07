// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

// TODO: Remove these when all the models are implemented
#![allow(dead_code)]

use std::fs::{self, File};
use std::io::{BufWriter, Write};

mod hashmap;
mod state_table;
mod range_coder;
mod models;

use range_coder::RangeCoder;
use state_table::StateTable;
use models::Model;
use models::order0::Order0;


// TODO: Add decompress method
fn main() {
    let timer = std::time::Instant::now();
    let file = "/data/calgary/book1";
    println!("Compressing {file}");

    let buf = fs::read(file).unwrap();
    let mut writer = BufWriter::new(File::create("book1.bin").unwrap());

    let state_table = StateTable::new();
    let mut model = Order0::init(&state_table);
    let mut coder = RangeCoder::new();

    writer.write_all(&buf.len().to_be_bytes()).unwrap();
    for byte in buf {
        for nib in [byte >> 4, byte % 15] {
            coder.encode4(&mut writer, nib, model.predict4(nib));
            model.update4(nib);
        }
    }

    coder.flush(&mut writer);
    println!("Took: {:?}", timer.elapsed());
}
