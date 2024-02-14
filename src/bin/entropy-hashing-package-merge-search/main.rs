use std::{io::Result, time::Instant};

use weath3rb0i::{encode8, entropy_coding::ArithmeticCoder, helpers::ACStats, models::Model};

mod model;
use model::PMHash;

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/book1")?;

    for history_size in 4..24 {
        for tree_depth in 7..16 {
            for meta_tree_depth in 7..16 {
            // for meta_tree_depth in [8] {
                let timer = Instant::now();
                let mut ac = ArithmeticCoder::new_coder();
                let mut model = PMHash::build(&buf, history_size, tree_depth, meta_tree_depth);
                let mut writer = ACStats::new();

                for byte in &buf {
                    encode8!(byte, bit, {
                        let p = model.predict();
                        model.update(bit);
                        ac.encode(bit, p, &mut writer)?;
                    });
                }

                ac.flush(&mut writer)?;
                let out_size = 0;
                let ratio = out_size as f64 / buf.len() as f64;
                println!("[ctx: {history_size:02}, tree: {tree_depth:02}, meta: {meta_tree_depth:02}] {out_size} ({ratio:.3}) in {:?}", timer.elapsed());
            }
        }
    }

    // TODO: add back compress & decompress with real IO so we can test
    // decompression still works
    Ok(())
}
