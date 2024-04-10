use std::{io::Result, time::Instant};

use weath3rb0i::{
    entropy_coding::{
        arithmetic_coder::ArithmeticCoder,
        package_merge::{canonical, package_merge},
    },
    helpers::{histogram, ACStats},
    models::{Model, OrderN},
    u64, u8,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    let levels = 2;
    let mut best = vec![u64!(buf.len()); levels];
    let mut params = vec![(0, 0); levels];

    for huffman_size in 7..=15 {
        best[1] = u64!(buf.len());
        params[1] = (0, 0);
        for ctx_bits in 8..=30 {
            let res = exec(&buf, huffman_size, ctx_bits)?;
            for i in 0..levels {
                if res > best[i] {
                    continue;
                }
                best[i] = res;
                params[i] = (huffman_size, ctx_bits);
            }
        }
        println!(
            "-> best: {} for [hsize: {}] when [ctx: {}, align: 0]",
            best[1], params[1].0, params[1].1
        );
    }
    println!(
        "-> gloabl best: {} for [hsize: {}, ctx: {}, align: 0]",
        best[0], params[0].0, params[0].1
    );

    Ok(())
}

fn exec(buf: &[u8], huffman_size: u8, ctx_bits: u8) -> Result<u64> {
    let (res, time) = (0..3)
        .map(|_| {
            let timer = Instant::now();
            let res = compress(buf, huffman_size, ctx_bits).unwrap();
            let time = timer.elapsed();
            (res, time)
        })
        .min_by(|(_, t1), (_, t2)| t1.cmp(t2))
        .unwrap();

    println!(
        "[ac-over-huff] [hsize: {:2}, ctx: {:2}, align: 0] csize: {} (ratio: {:.3}), ctime: {:?} ({:?} per bit)",
        huffman_size,
        ctx_bits,
        res,
        res as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );
    Ok(res)
}

fn compress(buf: &[u8], huffman_size: u8, ctx_bits: u8) -> Result<u64> {
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = OrderN::new(ctx_bits, 0);
    let mut writer = ACStats::new();

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
    Ok(writer.result())
}
