// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

// TODO: Remove these when all the models are implemented
#![allow(dead_code)]
#![allow(unused_imports)]
// #![deny(missing_docs)]

mod hashmap;
mod state_table;
mod arithmetic_coder;
mod models;
mod mixer;
mod smart_context;

use std::io::{BufReader, BufWriter, Write, Read};
use std::time::Instant;
use std::{env, fs, fs::File, path::PathBuf};

use models::{Model, Order0};
use arithmetic_coder::{ArithmeticCoder, BitWriter};

#[derive(Clone, Copy)]
enum Action {
    Compress,
    Decompress,
    Test
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        print_usage_and_panic("Invokation doesn't match usage! Provide 2 arguments.");
    }
    let path = PathBuf::from(&args[2]);
    let action = match args[1].as_str() {
        "c" => Action::Compress,
        "d" => Action::Decompress,
        "t" => Action::Test,
        _ => {
            print_usage_and_panic("Unrecognized option -> <action>!");
            unreachable!();
        }
    };

    if !path.is_file() && !path.is_dir() {
        panic!("Path must be a file or a directory!");
    }

    if path.is_dir() {
        for file in fs::read_dir(path)? {
            let file_path = file?.path();
            if file_path.is_file() {
                run(file_path, action)?;
            }
        }
    }
    else if path.is_file() {
        run(path, action)?;
    }

    Ok(())
}

fn run(file_path: PathBuf, action: Action) -> std::io::Result<()> {
    assert!(file_path.is_file());

    let mut out_path = std::env::current_dir()?;
    out_path.push(file_path.file_name().expect("Invalid file!"));

    let compress_path = out_path.with_extension("bin");
    let decompress_path = out_path.with_extension("orig");

    let timer = Instant::now();
    match action {
        Action::Compress => {
            compress(file_path, compress_path)?;
            println!("Compression took: {:?}", timer.elapsed());
        },
        Action::Decompress => {
            decompress(file_path, decompress_path)?;
            println!("Decompression took: {:?}", timer.elapsed());
        },
        Action::Test => {
            compress(file_path, compress_path.clone())?;
            println!("Compression took: {:?}", timer.elapsed());
            let timer = Instant::now();
            decompress(compress_path, decompress_path)?;
            println!("Decompression took: {:?}", timer.elapsed());
        }
    }

    Ok(())
}

fn compress(input_file: PathBuf, output_file: PathBuf) -> std::io::Result<()> {
    let reader = BufReader::new(File::open(input_file)?);
    let writer = BufWriter::new(File::create(output_file)?);

    let mut model = init_model();
    let mut ac = ArithmeticCoder::<_, BufReader<File>>::init_enc(writer);

    for byte_res in reader.bytes() {
        let byte = byte_res?;
        for nib in [byte >> 4, byte & 15] {
            let p4 = model.predict4(nib);
            model.update4(nib); // TODO: ctx.update4(nib) and model holds ref to ctx // FIXME:
            ac.encode4(nib, p4);
        }
    }

    ac.flush();
    Ok(())
}

fn decompress(input_file: PathBuf, output_file: PathBuf) -> std::io::Result<()> {
    let reader = BufReader::new(File::open(input_file).unwrap());
    let file_writer = BufWriter::new(File::create(output_file).unwrap());
    let mut writer = BitWriter::new(file_writer);

    let mut model = init_model();
    let mut ac = ArithmeticCoder::<BufWriter<File>, _>::init_dec(reader);


    // TODO: Refactor this loop
    loop {
        let p = model.predict();
        match ac.decode(p) {
            Ok(bit) => {
                model.update(bit);
                writer.bit_write_raw(bit);
            },
            _ => break
        }
    }
    writer.flush();
    Ok(())
}

// fn init_model() -> impl Model { // FIXME: Just hardcoding this for now, so the rust-analyzer can pick up some metadata
fn init_model() -> Order0 {
    Order0::init()
}

fn print_usage_and_panic(panic_msg: &str) {
    println!("Usage: weath3rb0i <Action> <Path>");
    println!("<Action> [single file]: c (compress), d (decompress), t (test = c + d)");
    println!("<Path> can be a single file or a directory");
    println!("Note: Directories are shallow traversed");
    panic!("{panic_msg}");
}

// TODO: Add tests
