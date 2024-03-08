use std::{io::Result, time::{Duration, Instant}};
use rayon::prelude::*;

use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    history::{ACHistoryCached, History},
    models::{ac_hash::Book1StationaryModel, Model, OrderNEntropy},
    u64, unroll_for,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    let levels = 3;
    let mut best = vec![(u64!(buf.len()), Duration::MAX); levels];
    let mut params = vec![(0, 0, 0); levels];

    for ctx_bits in 8..=26 {
        best[1] = (u64!(buf.len()), Duration::MAX);
        params[1] = (0, 0, 0);
        for alignment_bits in 0..=4 {
            best[2] = (u64!(buf.len()), Duration::MAX);
            params[2] = (0, 0, 0);
            let mut cache_sizes: Vec<u8> = (8..=24).collect();
            cache_sizes.insert(0, 0);
            let results: Vec<_> = cache_sizes.into_par_iter().map(|cache_size| {
                let model = Book1StationaryModel::new();
                let history = ACHistoryCached::new(ctx_bits - alignment_bits, model, cache_size);
                let results = exec(&buf, ctx_bits, alignment_bits, history, cache_size).unwrap();
                (cache_size, results)
            }).collect();
            for (cache_size, (res, time)) in results {
                for i in 0..levels {
                    if res > best[i].0 || (res == best[i].0 && time > best[i].1) {
                        continue;
                    }
                    best[i] = (res, time);
                    params[i] = (ctx_bits, alignment_bits, cache_size);
                }
            }
            println!(
                "-> fastest: {} in {:?} ({:?} per bit) for [ctx: {}, align: {}, cache: {}]",
                best[2].0, best[2].1, best[2].1.div_f64(buf.len() as f64 * 8.0), params[2].0, params[2].1, params[2].2
            );
        }
        println!(
            "--> best: {} in {:?} ({:?} per bit) for [ctx: {}, align: {}, cache: {}]",
            best[1].0, best[1].1, best[1].1.div_f64(buf.len() as f64 * 8.0), params[1].0, params[1].1, params[1].2
        );
    }
    println!(
        "--> gloabl best: {} in {:?} ({:?} per bit) for [ctx: {}, align: {}, cache: {}]",
        best[0].0, best[0].1, best[0].1.div_f64(buf.len() as f64 * 8.0), params[0].0, params[0].1, params[0].2
    );

    Ok(())
}

fn exec(buf: &[u8], ctx_bits: u8, alignment_bits: u8, history: impl History, cache_size: u8) -> Result<(u64, Duration)> {
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
        "[eh-ac] [ctx: {:2}, align: {} cache: {:2}] csize: {} (ratio {:.3}), ctime: {:?} ({:?} per bit)",
        ctx_bits,
        alignment_bits,
        cache_size,
        writer.result(),
        writer.result() as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );

    Ok((writer.result(), time))
}
