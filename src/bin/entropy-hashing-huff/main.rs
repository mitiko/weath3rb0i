use std::{io::Result, time::Instant};

use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    history::HuffHistory,
    models::{Model, OrderNEntropy},
    u64, unroll_for,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    let levels = 3;
    let mut best = vec![u64!(buf.len()); levels];
    let mut params = vec![(0, 0, 0); levels];

    for rem_huff_size in 7..=12 {
        best[1] = u64!(buf.len());
        params[1] = (0, 0, 0);
        for huff_size in 7..=15 {
            best[2] = u64!(buf.len());
            params[2] = (0, 0, 0);
            for ctx_bits in 8..=24 {
                let res = exec(&buf, huff_size, rem_huff_size, ctx_bits)?;
                for i in 0..levels {
                    if res > best[i] {
                        continue;
                    }
                    best[i] = res;
                    params[i] = (rem_huff_size, huff_size, ctx_bits);
                }
            }
            println!(
                "-> best: {} for [rem_hsize: {}, hsize: {}, ctx: {}]",
                best[2], params[2].0, params[2].1, params[2].2
            );
        }
        println!(
            "--> best: {} for [rem_hsize: {}, hsize: {}, ctx: {}]",
            best[1], params[1].0, params[1].1, params[1].2
        );
    }
    println!(
        "---> global best: {} for [rem_hsize: {}, hsize: {}, ctx: {}]",
        best[0], params[0].0, params[0].1, params[0].2
    );

    Ok(())
}

fn exec(buf: &[u8], huff_size: u8, rem_huff_size: u8, ctx_bits: u8) -> Result<u64> {
    let timer = Instant::now();
    let mut ac = ArithmeticCoder::new_coder();
    let history = HuffHistory::new(buf, huff_size, rem_huff_size);
    let mut model = OrderNEntropy::new(ctx_bits, 0, history);
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
        "[eh-huff] [rem_hsize: {:2}, hsize: {:2}, ctx: {:2}] csize: {} (ratio: {:.3}), ctime: {:?} ({:?} per bit)",
        rem_huff_size,
        huff_size,
        ctx_bits,
        writer.result(),
        writer.result() as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );

    Ok(writer.result())
}
