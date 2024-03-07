use std::io::Result;
use weath3rb0i::{models::Counter, unroll_for};

fn main() -> Result<()> {
    let buf = std::fs::read("/Users/mitiko/_data/enwik7")?;
    let mut model = [Counter::new(); 8];

    for byte in buf {
        let mut i = 7;
        unroll_for!(bit in byte, {
            i = (i + 1) & 7;
            model[i].update(bit);
        });
    }

    let res: Vec<u16> = model.iter().map(|c| c.p()).collect();
    println!("const PROB_TABLE: [u16; 8] = {:?};", res);

    Ok(())
}