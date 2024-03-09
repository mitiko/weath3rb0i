use std::{io::Result, time::Instant};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    history::{ACHistory, History},
    models::{ac_hash::{OrderNStationary, StationaryModel}, ACHashModel, Model, OrderNEntropy},
    u8, u64, unroll_for,
};

fn main() -> Result<()> {
    // _stat()?;
    // _run()?;

    _search()?;
    let _ = StationaryModel::for_book1();
    Ok(())
}

fn _stat() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    let orig_model = OrderNStationary::new(&buf, 11, 3);
    // let orig_model = StationaryModel::new(&buf);
    // let orig_model = StationaryModel::for_book1();
    // let orig_model = StationaryModel::for_enwik7();
    let mut model = orig_model.clone();

    let ctx_bits = 8.0;
    let align = 2;
    let distort = (8 - align) & 7;

    let mut total_entropy = 0.0;
    let mut total_gain = 0.0;
    // let data = b" the";
    // let data = b" the Brown Fox jumped over the fence";
    let data = &buf;
    let mut pos = 0;
    let flag = true;
    // flag = false -> 4.257
    // flag = true -> 7.245
    // conclusion: the model just sucks at small contexts.

    model.align(align);
    for (&b1, &b2) in data.iter().rev().skip(1).zip(data.iter().rev()) {
        let slice = u16::from_be_bytes([b1, b2]);
        let byte = u8!((slice >> distort) & 255); // distort
        let mut sum_gain = 0.0;
        let mut sum_entropy = 0.0;

        pos += 1;
        if (pos % 4 == 0) && flag {
            model.align(align);
        }
        for i in 0..8 {
            let bit = (byte >> i) & 1;
            let p = model.predict(bit);
            let p = p as f64 / (1 << 16) as f64;
            let entropy = -(p * p.log2() + (1.0 - p) * (1.0 - p).log2());
            let gain = if bit == 1 {
                - p.log2()
            } else {
                - (1.0 - p).log2()
            };
            // println!("bit {bit} of '{}' -> {:.3}, entropy = {:.3}, gain: {:.3}", byte as char, p, entropy, gain);
            sum_gain += gain;
            sum_entropy += entropy;
            if sum_gain > ctx_bits {
                model.align(align);
            }
        }
        // println!("[sum] entropy: {}, gain: {}", sum_entropy, sum_gain);
        total_entropy += sum_entropy;
        total_gain += sum_gain;
    }
    let n = data.len() as f64;
    println!("-> entropy: {total_entropy:.3}, avg: {:.3}", total_entropy / n);
    println!("-> gain: {total_gain:.3}, avg: {:.3}", total_gain / n);

    Ok(())
}

fn _run() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    let model = OrderNStationary::new(&buf, 11, 3);
    // let model = StationaryModel::for_book1();

    let history = ACHistory::new(8, model.clone());
    _exec(&buf, 8, 3, history).unwrap();
    let history = ACHistory::new(16, model.clone());
    _exec(&buf, 16, 3, history).unwrap();
    Ok(())
}

fn _search() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    // let buf = std::fs::read("/Users/mitiko/_data/enwik7")?;

    let levels = 4;
    let mut best = vec![u64!(buf.len()); levels];
    let mut params = vec![(0, 0, 0, 0); levels];

    // for inner_ctx_bits in 0..=16 {
    for inner_ctx_bits in [8, 16] {
        best[1] = u64!(buf.len());
        params[1] = (0, 0, 0, 0);
        // for inner_alignment_bits in 0..=4 {
        for inner_alignment_bits in [6, 7] {
            best[2] = u64!(buf.len());
            params[2] = (0, 0, 0, 0);
            let model = OrderNStationary::new(
                &buf,
                inner_ctx_bits + inner_alignment_bits,
                inner_alignment_bits,
            );

            // for ctx_bits in 8..=26 {
            for ctx_bits in [8, 16] {
                best[3] = u64!(buf.len());
                params[3] = (0, 0, 0, 0);
                let results: Vec<_> = (0..=4)
                    .into_par_iter()
                    .map(|alignment_bits| {
                        let history = ACHistory::new(ctx_bits - alignment_bits, model.clone());
                        let result = _exec(&buf, ctx_bits, alignment_bits, history).unwrap();
                        (result, alignment_bits)
                    })
                    .collect();
                for (res, alignment_bits) in results {
                    for i in 0..levels {
                        if res > best[i] {
                            continue;
                        }
                        best[i] = res;
                        params[i] = (
                            ctx_bits,
                            alignment_bits,
                            inner_ctx_bits,
                            inner_alignment_bits,
                        );
                    }
                }
                println!(
                    "-> best: {} for [ctx: {}, align: {}, _ctx: {}, _align: {}]",
                    best[3], params[3].0, params[3].1, params[3].2, params[3].3
                );
            }
            println!(
                "--> best: {} for [ctx: {}, align: {}, _ctx: {}, _align: {}]",
                best[2], params[2].0, params[2].1, params[2].2, params[2].3
            );
        }
        println!(
            "---> best: {} for [ctx: {}, align: {}, _ctx: {}, _align: {}]",
            best[1], params[1].0, params[1].1, params[1].2, params[1].3
        );
    }
    println!(
        "-> gloabl best: {} for [ctx: {}, align: {}, _ctx: {}, _align: {}]",
        best[0], params[0].0, params[0].1, params[0].2, params[0].3
    );

    Ok(())
}

fn _exec(buf: &[u8], ctx_bits: u8, alignment_bits: u8, history: impl History) -> Result<u64> {
    let timer = Instant::now();
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = OrderNEntropy::new(ctx_bits, alignment_bits, history);
    let mut writer = ACStats::new();

    for byte in buf {
        unroll_for!(bit in byte, {
            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;

    let time = timer.elapsed();
    println!(
        "[eh-ac-on] [ctx: {:2}, align: {}] csize: {} (ratio {:.3}), ctime: {:?} ({:?} per bit)",
        ctx_bits,
        alignment_bits,
        writer.result(),
        writer.result() as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );

    Ok(writer.result())
}
