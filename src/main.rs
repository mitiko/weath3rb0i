// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

// TODO: Remove these when all the models are implemented
#![allow(dead_code)]
#![allow(unused_imports)]

use std::fs::{self, File};
use std::io::{BufWriter, Write, BufReader, Read};

mod analyzers;
mod hashmap;
mod state_table;
mod range_coder;
mod models;
mod mixer;

use range_coder::RangeCoder;
use state_table::StateTable;
use models::Model;
use models::order0::Order0;
use models::order1::Order1;
use mixer::Mixer2;
use analyzers::alphabet_reordering::AlphabetOrderManager;


// TODO: Add decompress method
fn main() {
    let timer = std::time::Instant::now();
    let file = "/data/calgary/book1";
    println!("Compressing {file}");

    let buf = fs::read(file).unwrap();
    let mut writer = BufWriter::new(File::create("book1.bin").unwrap());

    let mut model0 = Order0::init();
    let mut coder = RangeCoder::new();

    writer.write_all(&(buf.len() as u64).to_be_bytes()).unwrap();
    for byte in buf {
        for nib in [byte >> 4, byte & 15] {
            let p = model0.predict4(nib);
            coder.encode4(&mut writer, nib, p);
            model0.update4(nib);
        }
    }

    coder.flush(&mut writer);
    println!("Took: {:?}", timer.elapsed());
    decode();
}

fn decode() {
    let timer = std::time::Instant::now();
    let file = "book1.bin";
    println!("Decompressing {file}");

    let mut reader = BufReader::new(File::open(file)         .unwrap());
    let mut writer = BufWriter::new(File::create("book1.dec").unwrap());

    let mut model0 = Order0::init();
    let mut coder = RangeCoder::new();

    let mut buf = [0; 256];
    reader.read(&mut buf[..8]).unwrap();
    let size = u64::from_be_bytes(buf[..8].try_into().unwrap());
    reader.read(&mut buf[..4]).unwrap();
    coder.init_decode(u32::from_be_bytes(buf[..4].try_into().unwrap()));
    let mut written = 0;

    loop {
        let mut byte = 1;
        let mut eof = false;
        while byte < 256 {
            let p = model0.predict();
            let bit = coder.decode(p);
            model0.update(bit);
            eof = coder.renorm_dec(&mut reader);
            byte = (byte * 2) + bit as usize;
        }
        byte -= 256;
        writer.write(&[byte as u8]).unwrap(); written += 1;
        if written == size || eof { break; }
    }
    writer.flush().unwrap();
    println!("Took: {:?}", timer.elapsed());
}
