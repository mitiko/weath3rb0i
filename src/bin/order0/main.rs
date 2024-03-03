use std::{io::Result, time::Instant};

use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    models::{Model, Order0},
    unroll_for,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    let timer = Instant::now();
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = Order0::new();
    let mut writer = ACStats::new();

    for byte in &buf {
        unroll_for!(bit in byte, {
            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;

    println!(
        "[order0] csize: {} (ratio: {:.3}), ctime: {:?}",
        writer.result(),
        writer.result() as f64 / buf.len() as f64,
        timer.elapsed()
    );

    Ok(())
}
