use std::fs;
use std::io;

use weath3rb0i::entropy_coding::ArithmeticCoder;
use weath3rb0i::helpers::ACStats;
use weath3rb0i::mixers::*;
use weath3rb0i::models::Model;
use weath3rb0i::models::{Composite2, Counter4, PrefixModel16, PrefixModel8};
use weath3rb0i::unroll_for;

/*
| mixer               | book1  | time  |
|---------------------|--------|-------|
| EntropyWeightMixer2 | 361191 | 166ms |
| AbsWeightMixer2     | 365570 | 79ms  |
| ConfidenceMixer2    | 368043 | 98ms  |
| MeanMixer2          | 530858 | 76ms  |

models: PM8, PM16
cpu: Apple M5
*/

fn main() -> io::Result<()> {
    let buf = fs::read("/Users/mitiko/_data/calgary/book1")?;

    let (a, t) = run(&buf, build_model(ConfidenceMixer2));
    println!("ConfidenceMixer2\t{}, {:?}", a, t);
    let (b, t) = run(&buf, build_model(MeanMixer2));
    println!("MeanMixer2\t\t{}, {:?}", b, t);
    let (c, t) = run(&buf, build_model(AbsWeightMixer2));
    println!("AbsWeightMixer2\t\t{}, {:?}", c, t);
    let (d, t) = run(&buf, build_model(EntropyWeightMixer2));
    println!("EntropyWeightMixer2\t{}, {:?}", d, t);

    Ok(())
}

fn build_model(mixer: impl Mixer2) -> impl Model {
    let m1 = PrefixModel8::new(Counter4::new());
    let m2 = PrefixModel16::new(Counter4::new());
    Composite2::new(m1, m2, mixer)
}

fn run(buf: &[u8], mut model: impl Model) -> (u64, std::time::Duration) {
    let mut ac = ArithmeticCoder::new_coder();
    let mut stats = ACStats::new();
    let timer = std::time::Instant::now();
    for byte in buf.iter() {
        unroll_for!(bit in byte, {
            _ = ac.encode(bit, model.predict(), &mut stats);
            model.update(bit);
        });
    }
    (stats.result(), timer.elapsed())
}
