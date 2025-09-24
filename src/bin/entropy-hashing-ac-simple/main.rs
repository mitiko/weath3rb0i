use std::{io::Result, time::Instant};

use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    history::{ACHistory, History},
    models::{Model, Order0, Order1, OrderNEntropy},
    u64, unroll_for,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    // let buf = std::fs::read("/Users/mitiko/_data/enwik7")?;

    // TODO: add generic parameter optimizer
    let levels = 2;
    let mut best = vec![u64!(buf.len()); levels];
    let mut params = vec![(0, 0); levels];

    for ctx_bits in 8..=30 {
        best[1] = u64!(buf.len());
        params[1] = (0, 0);
        for alignment_bits in 0..=4 {
            let model = Order1::new();
            let _model = Order0::new();
            let history = ACHistory::new(model);
            let res = exec(&buf, ctx_bits, alignment_bits, history)?;
            for i in 0..levels {
                if res > best[i] {
                    continue;
                }
                best[i] = res;
                params[i] = (ctx_bits, alignment_bits);
            }
        }
        println!(
            "-> best: {} for [ctx: {}, align: {}]",
            best[1], params[1].0, params[1].1
        );
    }
    println!(
        "-> gloabl best: {} for [ctx: {}, align: {}]",
        best[0], params[0].0, params[0].1
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
        "[eh-acs] [ctx: {:2}, align: {}] csize: {} (ratio {:.3}), ctime: {:?} ({:?} per bit)",
        ctx_bits,
        alignment_bits,
        writer.result(),
        writer.result() as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );

    Ok(writer.result())
}
