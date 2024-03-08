use std::{io::Result, time::Instant};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    history::{ACHistory, History},
    models::{ac_hash::OrderNStationary, ACHashModel, Model, OrderNEntropy},
    u64, unroll_for,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    // let buf = std::fs::read("/Users/mitiko/_data/enwik7")?;

    let levels = 4;
    let mut best = vec![u64!(buf.len()); levels];
    let mut params = vec![(0, 0, 0, 0); levels];

    // for inner_ctx_bits in 0..=16 {
    for inner_ctx_bits in [8] {
        best[1] = u64!(buf.len());
        params[1] = (0, 0, 0, 0);
        // for inner_alignment_bits in 0..=4 {
        for inner_alignment_bits in [3] {
            best[2] = u64!(buf.len());
            params[2] = (0, 0, 0, 0);
            let model = OrderNStationary::new(&buf, inner_ctx_bits + inner_alignment_bits, inner_alignment_bits);

            for ctx_bits in 8..=26 {
                best[3] = u64!(buf.len());
                params[3] = (0, 0, 0, 0);
                let results: Vec<_> = (0..=4).into_par_iter().map(|alignment_bits| {
                    let history = ACHistory::new(ctx_bits - alignment_bits, model.clone());
                    let result = exec(&buf, ctx_bits, alignment_bits, history).unwrap();
                    (result, alignment_bits)
                }).collect();
                for (res, alignment_bits) in results {
                    for i in 0..levels {
                        if res > best[i] {
                            continue;
                        }
                        best[i] = res;
                        params[i] = (ctx_bits, alignment_bits, inner_ctx_bits, inner_alignment_bits);
                    }
                }
                println!(
                    "-> best: {} for [ctx: {}, align: {}, _ctx: {}, _align: {}]",
                    best[3], params[3].0, params[3].1, params[3].2, params[3].3
                );
            }
            println!(
                "-> best: {} for [ctx: {}, align: {}, _ctx: {}, _align: {}]",
                best[2], params[2].0, params[2].1, params[2].2, params[2].3
            );
        }
        println!(
            "-> best: {} for [ctx: {}, align: {}, _ctx: {}, _align: {}]",
            best[1], params[1].0, params[1].1, params[1].2, params[1].3
        );
    }
    println!(
        "-> gloabl best: {} for [ctx: {}, align: {}, _ctx: {}, _align: {}]",
        best[0], params[0].0, params[0].1, params[0].2, params[0].3
    );

    Ok(())
}

fn exec(buf: &[u8], ctx_bits: u8, alignment_bits: u8, history: impl History) -> Result<u64> {
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
        "[eh-ac] [ctx: {:2}, align: {}] csize: {} (ratio {:.3}), ctime: {:?} ({:?} per bit)",
        ctx_bits,
        alignment_bits,
        writer.result(),
        writer.result() as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );

    Ok(writer.result())
}
