use std::{io::Result, time::Instant};
use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    models::{Model, OrderN},
    u64, unroll_for,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    let levels = 2;
    let mut best = vec![u64!(buf.len()); levels];
    let mut params = vec![(0, 0); levels];

    for ctx_bits in 8..=30 {
        best[1] = u64!(buf.len());
        params[1] = (0, 0);
        for alignment_bits in 0..=4 {
            let res = exec(&buf, ctx_bits, alignment_bits)?;
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

fn exec(buf: &[u8], ctx_bits: u8, alignment_bits: u8) -> Result<u64> {
    let (res, time) = (0..3)
        .map(|_| {
            let timer = Instant::now();
            let res = compress(buf, ctx_bits, alignment_bits).unwrap();
            let time = timer.elapsed();
            (res, time)
        })
        .min_by(|(_, t1), (_, t2)| t1.cmp(t2))
        .unwrap();

    println!(
        "[ordern] [ctx: {:2}, align: {}] csize: {} (ratio: {:.3}), ctime: {:?} ({:?} per bit)",
        ctx_bits,
        alignment_bits,
        res,
        res as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );

    Ok(res)
}

fn compress(buf: &[u8], ctx_bits: u8, alignment_bits: u8) -> Result<u64> {
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = OrderN::new(ctx_bits, alignment_bits);
    let mut writer = ACStats::new();

    for byte in buf {
        unroll_for!(bit in byte, {
            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;
    Ok(writer.result())
}
