use std::{
    fs::File,
    io::{BufWriter, Result, Write},
    time::Instant,
};

use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder,
    helpers::ACStats,
    history::{ACHistory, History},
    models::{FrozenModel, Model, Order0, Order0Generic},
    unroll_for, Analytics,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;
    // let buf = std::fs::read("/Users/mitiko/_data/enwik7")?;

    let mut model = FrozenModel::new(Order0::new());
    model.train(&buf);
    let history = ACHistory::new(12, model.clone());
    let _res = exec(&buf, history, "analytics.jsonl")?;

    Ok(())
}

fn exec<H: History + Analytics>(buf: &[u8], history: H, analytics_path: &str) -> Result<u64> {
    let timer = Instant::now();
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = Order0Generic::new(history);
    let mut writer = ACStats::new();
    // one line per encoded bit, so buffer generously to stay off the syscall path
    let mut analytics = BufWriter::with_capacity(1 << 22, File::create(analytics_path)?);

    // first line is the metadata header, the rest are per-bit logs
    writeln!(analytics, "{}", Order0Generic::<H>::metadata())?;

    for byte in buf {
        unroll_for!(bit in byte, {
            writeln!(analytics, "{}", model.log())?;

            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;
    analytics.flush()?;

    let time = timer.elapsed();
    println!(
        "[eh-ac] csize: {} (ratio {:.3}), ctime: {:?} ({:?} per bit)",
        writer.result(),
        writer.result() as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );

    Ok(writer.result())
}
