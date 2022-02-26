// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

// TODO: Remove these when all the models are implemented
#![allow(dead_code)]
// #![allow(unused_imports)]
// #![deny(missing_docs)]

mod analyzers;
mod hashmap;
mod state_table;
mod range_coder;
mod models;
mod mixer;
mod runner;

use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    compress: Option<PathBuf>,
    #[clap(short, long)]
    decompress: Option<PathBuf>
}

fn main() {
    let cli = Cli::parse();
    if matches!(cli.compress, Some(ref file_path) if file_path.is_file()) {
        let input_path = cli.compress.unwrap();
        let mut output_path = std::env::current_dir().unwrap();
        output_path.push(input_path.file_name().unwrap());
        output_path.set_extension("bin");

        let timer = std::time::Instant::now();
        println!("Compressing {input_path:?}");
        runner::encode(&input_path, &output_path);
        println!("Took: {:?} -> {}", timer.elapsed(), output_path.metadata().unwrap().len());
    }
    else if matches!(cli.decompress, Some(ref file_path) if file_path.is_file()) {
        let input_path = cli.decompress.unwrap();
        let mut output_path = std::env::current_dir().unwrap();
        output_path.push(input_path.file_name().unwrap());
        output_path.set_extension("dec");

        let timer = std::time::Instant::now();
        println!("Decompressing {input_path:?}");
        runner::decode(&input_path, &output_path);
        println!("Took: {:?} -> {}", timer.elapsed(), output_path.metadata().unwrap().len());
    }
    else if cli.compress.or(cli.decompress).is_none() {
        let timer = std::time::Instant::now();
        println!("Compressing default=book1");
        runner::encode(&PathBuf::from("/data/calgary/book1"), &PathBuf::from("./book1.bin"));
        println!("Took: {:?} -> {}", timer.elapsed(), std::fs::metadata("./book1.bin").unwrap().len());

        let timer = std::time::Instant::now();
        println!("Decompressing");
        runner::decode(&PathBuf::from("./book1.bin"),         &PathBuf::from("./book1.dec"));
        println!("Took: {:?} -> {}", timer.elapsed(), std::fs::metadata("./book1.dec").unwrap().len());
    }
}

// TODO: Add tests
