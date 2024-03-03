use std::{io::Result, time::Instant};

use weath3rb0i::{
    entropy_coding::{
        package_merge::{canonical, package_merge},
        arithmetic_coder::ArithmeticCoder,
    },
    helpers::{histogram, ACStats},
    models::{Model, Order0},
    u64, u8,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    let mut best = u64!(buf.len());
    let mut params = 0;
    for huffman_size in 7..16 {
        let res = exec(&buf, huffman_size)?;
        if res < best {
            params = huffman_size;
            best = res;
        }
    }
    println!("best: {best} for [hsize: {params}]"); // TODO: color

    Ok(())
}

fn exec(buf: &[u8], huffman_size: u8) -> Result<u64> {
    let timer = Instant::now();
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = Order0::new();
    let mut writer = ACStats::new(); // TODO: order n?

    let counts = histogram(&buf);
    let code_lens = package_merge(&counts, huffman_size);
    let huffman = canonical(&code_lens);

    for &byte in buf {
        let (code, len) = huffman[usize::from(byte)];
        for i in (0..len).rev() {
            let p = model.predict();
            let bit = u8!((code >> i) & 1);
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        }
    }
    ac.flush(&mut writer)?;

    println!(
        "[ac-over-huffman] [hsize: {:02} b{:02}] csize: {} (ratio: {:.3}), ctime: {:?}",
        huffman_size,
        8, // bits in context
        writer.result(),
        writer.result() as f64 / buf.len() as f64,
        timer.elapsed()
    );

    Ok(writer.result())
}
