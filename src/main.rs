// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

// TODO: Remove these when all the models are implemented
#![allow(dead_code)]
#![allow(unused_imports)]
// #![deny(missing_docs)]

mod arithmetic_coder;
mod bit_helpers;
mod hashmap;
mod mixer;
mod models;
mod smart_context;
mod state_table;

use std::io::{BufReader, BufWriter, Read, Write};
use std::time::Instant;
use std::{env, fs, fs::File, path::PathBuf};

#[macro_use]
extern crate debug_unreachable;

use arithmetic_coder::ArithmeticCoder;
use bit_helpers::BitWriter;
use models::{Model, Order0};

const MAGIC_STR: &[u8; 4] = b"0000";
const MAGIC_NUM: u32 = u32::from_be_bytes(*MAGIC_STR);

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
            unsafe { debug_unreachable!(); } // we've already panicked
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
    } else if path.is_file() {
        run(path, action)?;
    }

    Ok(())
}

fn run(file_path: PathBuf, action: Action) -> std::io::Result<()> {
    assert!(file_path.is_file());

    let out_path = {
        let mut out_path = std::env::current_dir()?;
        out_path.push(file_path.file_name().unwrap());

        match action {
            Action::Compress | Action::Test => out_path.set_extension("bin"),
            Action::Decompress => out_path.set_extension("orig")
        };

        out_path
    };


    let timer = Instant::now();
    match action {
        Action::Compress => {
            compress(file_path, out_path)?;
            println!("Compression took: {:?}", timer.elapsed());
        }
        Action::Decompress => {
            decompress(file_path, out_path)?;
            println!("Decompression took: {:?}", timer.elapsed());
        }
        Action::Test => {
            run(file_path, Action::Compress)?;
            run(out_path, Action::Decompress)?;
        }
    }

    Ok(())
}

fn compress(input_file: PathBuf, output_file: PathBuf) -> std::io::Result<()> {
    let mut writer = BufWriter::new(File::create(output_file)?);
    let reader = {
        let f = File::open(input_file)?;
        let len = f.metadata()?.len();

        writer.write_all(MAGIC_STR)?;
        writer.write_all(&len.to_be_bytes())?;
        BufReader::new(f)
    };
    let mut ac = ArithmeticCoder::<_, BufReader<File>>::init_enc(writer);
    let mut model = init_model();
    
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
    let mut reader = BufReader::new(File::open(input_file).unwrap());
    let mut writer = {
        let buf_writer = BufWriter::new(File::create(output_file).unwrap());
        BitWriter::new(buf_writer)
    };
    
    let len = {
        let mut len_buf = [0; std::mem::size_of::<u32>() + std::mem::size_of::<u64>()];
        reader.read_exact(&mut len_buf)?;

        let magic_num = u32::from_be_bytes(len_buf[..4].try_into().unwrap());
        assert_eq!(magic_num, MAGIC_NUM, "Magic numbers don't match up - file wasn't compressed with (this version of) weath3rb0i!");
        u64::from_be_bytes(len_buf[4..].try_into().unwrap())
    };
    let mut ac = ArithmeticCoder::<BufWriter<File>, _>::init_dec(reader);
    let mut model = init_model();


    for _ in 0..len {
        for _ in 0..8 {
            let p = model.predict();
            let bit = ac.decode(p);
            writer.write_bit(bit);
            model.update(bit);
        }
    }

    writer.try_flush();
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
