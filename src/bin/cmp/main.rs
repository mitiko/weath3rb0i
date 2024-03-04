use std::{
    fs::File,
    io::{BufWriter, Result},
    time::Instant,
};

use weath3rb0i::{
    entropy_coding::{arithmetic_coder::ArithmeticCoder, io::ACWriter},
    helpers::cmp,
    models::{Model, Order0, OrderN},
    unroll_for,
};

fn main() -> Result<()> {
    let model = Order0::new();
    exec(model, "order0")?;

    let model = OrderN::new(11, 3);
    exec(model, "ordern")?;

    cmp("order0.bin", "ordern.bin")?;

    Ok(())
}

fn exec(mut model: impl Model, name: &str) -> Result<()> {
    let timer = Instant::now();
    let buf = std::fs::read("/Users/mitiko/_data/enwik7")?;
    let mut ac = ArithmeticCoder::new_coder();
    let file = File::create(format!("{}.bin", name))?;
    let mut writer = ACWriter::new(BufWriter::new(&file));

    for byte in &buf {
        unroll_for!(bit in byte, {
            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;

    let res = File::metadata(&file)?.len();
    let time = timer.elapsed();
    println!(
        "[{}] csize: {} (ratio: {:.3}), ctime: {:?} ({:?} per bit)",
        name,
        res,
        res as f64 / buf.len() as f64,
        time,
        time.div_f64(buf.len() as f64 * 8.0)
    );

    Ok(())
}
