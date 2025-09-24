use std::{io::Result, time::Instant};

use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    history::{ACHistory, History},
    models::{Model, Order0, OrderNEntropy, StaticOrder0},
    u64, unroll_for,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    // optimize(&buf)?;
    show_steps(&buf);

    Ok(())
}

fn build_model(buf: &[u8]) -> StaticOrder0 {
    let mut model = Order0::new();
    for byte in buf {
        unroll_for!(bit in byte, {
            model.update(bit);
        });
    }
    StaticOrder0::new(model)
}

#[allow(dead_code)]
fn show_steps(buf: &[u8]) {
    let model = build_model(buf);
    let mut history = ACHistory::new(model);

    for byte in buf.iter().skip(100).take(10) {
        unroll_for!(bit in byte, {
            history.update(bit);
            let hash = history.hash();
            let repr = match byte {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => *byte as char,
                _ => '.',
            };
            println!("{bit} of {byte:08b} {repr} -> {:032b}", hash);
        });
    }
}

#[allow(dead_code)]
fn optimize(buf: &[u8]) -> Result<()> {
    // static model: csize: 387461 (ratio 0.504), ctime: 82.277083ms (13ns per bit)
    // TODO: add generic parameter optimizer
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
    let model = build_model(buf);
    let history = ACHistory::new(model);

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
