// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0
use std::io::{BufReader, BufWriter, Read, Write};
use std::time::Instant;
use std::{env, fs, fs::File, path::PathBuf};

use weath3rb0i::models::{Model, SmartCtx, SharedCtx};
use weath3rb0i::debug_unreachable;
use weath3rb0i::entropy_coding::{ArithmeticCoder, ac_io::{ACWriter, ACReader}};

const MAGIC_STR: &[u8; 4] = b"w30i";
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
        print_usage_and_exit("Invokation doesn't match usage! Provide 2 arguments.");
    }
    let path = PathBuf::from(&args[2]);
    let action = match args[1].as_str() {
        "c" => Action::Compress,
        "d" => Action::Decompress,
        "t" => Action::Test,
        _ => {
            print_usage_and_exit("Unrecognized option -> <action>!");
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
    let mut ac = ArithmeticCoder::<_>::new_coder(ACWriter::new(writer));
    let mut ctx = SmartCtx::new();
    let mut model = init_model();

    for byte in reader.bytes() {
        let byte = byte?;
        for nib in [byte >> 4, byte & 15] {
            let p4 = model.predict4(&ctx, nib);
            model.update4(&ctx, nib);
            ctx.update4(nib);
            ac.encode4(nib, p4)?;
        }
    }

    ac.flush()?;
    Ok(())
}

fn decompress(input_file: PathBuf, output_file: PathBuf) -> std::io::Result<()> {
    let mut reader = BufReader::new(File::open(input_file)?);
    let mut writer = BufWriter::new(File::create(output_file)?);

    let len = {
        let mut len_buf = [0; std::mem::size_of::<u32>() + std::mem::size_of::<u64>()];
        reader.read_exact(&mut len_buf)?;

        let magic_num = u32::from_be_bytes(len_buf[..4].try_into().unwrap());
        assert_eq!(magic_num, MAGIC_NUM, "Magic numbers don't match up - file wasn't compressed with (this version of) weath3rb0i!");
        u64::from_be_bytes(len_buf[4..].try_into().unwrap())
    };
    let mut ac = ArithmeticCoder::<_>::new_decoder(ACReader::new(reader))?;
    let mut ctx = SmartCtx::new();
    let mut model = init_model();

    for _ in 0..len {
        let mut byte = 0;
        for _ in 0..u8::BITS {
            let p = model.predict(&ctx);
            let bit = ac.decode(p)?;
            model.update(&ctx, bit);
            ctx.update(bit);
            byte = (byte << 1) | bit;
        }
        writer.write_all(&[byte])?;
    }

    writer.flush()?;
    Ok(())
}

// use weath3rb0i::models::Order0;
// fn init_model() -> Order0 {
//     Order0::init()
// }

use weath3rb0i::models::TinyOrder0;
fn init_model() -> TinyOrder0 {
    TinyOrder0::init()
}

fn print_usage_and_exit(msg: &str) {
    println!("Usage: weath3rb0i <Action> <Path>");
    println!("<Action> [single file]: c (compress), d (decompress), t (test = c + d)");
    println!("<Path> can be a single file or a directory");
    println!("Note: Directories are shallow traversed");
    println!("\n{}", msg);
    std::process::exit(1);
}

// TODO: Add tests
