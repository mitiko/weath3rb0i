use std::io::{BufReader, BufWriter, Read, Write};
use std::time::Instant;
use std::{env, fs::File, path::PathBuf};

use weath3rb0i::helpers::{cmp, get_len, read_u64};
use weath3rb0i::{entropy_coding::*, models::*, unroll_for};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Compress,
    Decompress,
    Test,
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        print_usage_and_exit("Invokation doesn't match usage!");
    }
    let action = match args[1].as_str() {
        "c" => Action::Compress,
        "d" => Action::Decompress,
        "t" => Action::Test,
        _ => print_usage_and_exit("Unrecognized option -> <action>!"),
    };
    let in_path = PathBuf::from(&args[2]);
    let out_path = PathBuf::from(&args[3]);

    run(in_path, out_path, action)?;

    Ok(())
}

fn run(in_path: PathBuf, out_path: PathBuf, action: Action) -> std::io::Result<()> {
    assert!(in_path.is_file());

    let timer = Instant::now();
    match action {
        Action::Compress => {
            compress(in_path, out_path)?;
            println!("Compression took: {:?}", timer.elapsed());
        }
        Action::Decompress => {
            decompress(in_path, out_path)?;
            println!("Decompression took: {:?}", timer.elapsed());
        }
        Action::Test => {
            let orig_path = in_path.clone().with_added_extension("orig");
            run(in_path.clone(), out_path.clone(), Action::Compress)?;
            run(out_path, orig_path.clone(), Action::Decompress)?;
            cmp(&in_path.to_string_lossy(), &orig_path.to_string_lossy())?;
        }
    }

    Ok(())
}

fn compress(input_file: PathBuf, output_file: PathBuf) -> std::io::Result<()> {
    let mut writer = BufWriter::new(File::create(output_file)?);
    let (mut reader, len) = get_len(input_file);
    writer.write_all(&len.to_be_bytes())?;

    let mut writer = ACWriter::new(writer);
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = init_model(&mut reader);

    for byte in reader.bytes().map(|byte| byte.unwrap()) {
        unroll_for!(bit in byte, {
            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }

    ac.flush(&mut writer)?;
    Ok(())
}

fn decompress(input_file: PathBuf, output_file: PathBuf) -> std::io::Result<()> {
    let mut reader = BufReader::new(File::open(input_file)?);
    let mut writer = BufWriter::new(File::create(output_file)?);
    let len = read_u64(&mut reader)?;

    let mut model = read_model(&mut reader);
    let mut reader = ACReader::new(reader);
    let mut ac = ArithmeticCoder::new_decoder(&mut reader)?;

    for _ in 0..len {
        let mut byte = 0;
        for _ in 0..u8::BITS {
            let p = model.predict();
            let bit = ac.decode(p, &mut reader)?;
            model.update(bit);
            byte = (byte << 1) | bit;
        }
        writer.write_all(&[byte])?;
    }

    writer.flush()?;
    Ok(())
}

fn init_model(_reader: &mut impl Read) -> impl Model {
    PrefixModel8::new(Counter4::new())
}

fn read_model(_reader: &mut impl Read) -> impl Model {
    PrefixModel8::new(Counter4::new())
}

fn print_usage_and_exit(msg: &str) -> ! {
    println!("Usage: weath3rb0i <Action> <Path>");
    println!("<Action> [single file]: c (compress), d (decompress), t (test = c + d)");
    println!("<Path> can be a single file or a directory");
    println!("Note: Directories are shallow traversed");
    println!("\n{}", msg);
    std::process::exit(1);
}
