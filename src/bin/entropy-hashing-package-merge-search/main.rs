use std::{io::Result, time::Instant};

mod model;
use model::PMHash;
use weath3rb0i::{
    entropy_coding::arithmetic_coder::ArithmeticCoder, helpers::ACStats, models::Model, unroll_for,
};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    for history_size in 4..24 {
        for tree_depth in 7..16 {
            for meta_tree_depth in 7..16 {
                // for meta_tree_depth in [8] {
                exec(&buf, history_size, tree_depth, meta_tree_depth)?;
            }
        }
    }

    // TODO: add back compress & decompress with real IO so we can test
    // decompression still works
    Ok(())
}

fn exec(buf: &[u8], history_size: u8, tree_depth: u8, meta_tree_depth: u8) -> Result<()> {
    let timer = Instant::now();
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = PMHash::build(&buf, history_size, tree_depth, meta_tree_depth);
    let mut writer = ACStats::new();

    for byte in buf {
        unroll_for!(bit in byte, {
            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }
    ac.flush(&mut writer)?;

    let out_size = 0;
    let ratio = out_size as f64 / buf.len() as f64;
    println!(
        "[eh-pm] {:02}, tree: {:02}, meta: {:02}] {} ({:.3}) in {:?}",
        history_size,
        tree_depth,
        meta_tree_depth,
        out_size,
        ratio,
        timer.elapsed()
    );

    Ok(())
}
