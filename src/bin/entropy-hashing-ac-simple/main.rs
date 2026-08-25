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
    let _res = exec(&buf, history, "analytics")?;

    Ok(())
}

/// Writes two files: `<out>.jsonl` holds the metadata header and the full model state
/// before every bit, `<out>.p16` holds just the probability each bit was coded with, as
/// little-endian u16. The sidecar is what a reader needs to compute cost per bit, and it
/// avoids having to parse gigabytes of JSON to find one field.
fn exec<H: History + Analytics>(buf: &[u8], history: H, out: &str) -> Result<u64> {
    let timer = Instant::now();
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = Order0Generic::new(history);
    let mut writer = ACStats::new();
    // one line per encoded bit, so buffer generously to stay off the syscall path
    let mut states = BufWriter::with_capacity(1 << 22, File::create(format!("{out}.jsonl"))?);
    let mut probs = BufWriter::with_capacity(1 << 16, File::create(format!("{out}.p16"))?);

    // first line is the metadata header, the rest are per-bit logs
    writeln!(states, "{}", model.metadata())?;

    for byte in buf {
        unroll_for!(bit in byte, {
            writeln!(states, "{}", model.log())?;

            let p = model.predict();
            probs.write_all(&p.to_le_bytes())?;
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;
    states.flush()?;
    probs.flush()?;

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
