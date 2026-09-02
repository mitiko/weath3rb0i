use std::{fs, io};
use weath3rb0i::{
    entropy_coding::ArithmeticCoder,
    helpers::ACStats,
    history::{ACHistory, History, HistoryBitOrder},
    models::{
        counters::*, CtxModel, CtxPrefixModel13, CtxPrefixModel16, CtxPrefixModel8, FreezeModel,
        PrefixModel8,
    },
    unroll_for, usize,
};

/*
| model     | LSB    | MSB    | Boundary |
|-----------|--------|--------|----------|
| tree 8    | 457308 | 422957 | 444776   |
| suffix 8  | 461637 | 411411 | 434803   |
| tree 13   | 365566 | 311173 | 335762   |
| suffix 13 | 362180 | 304601 | 329846   |
| tree 16   | 322759 | 279366 | 292386   |
| suffix 16 | 313162 | 269544 | 284930   |
*/

fn main() -> io::Result<()> {
    let buf = fs::read("/Users/mitiko/_data/calgary/book1")?;

    println!("training inner model...");
    let mut model = PrefixModel8::new(Counter4::new());
    model.train(&buf);
    let inner = model.freeze();

    println!(
        "{:<12} {:>12} {:>12} {:>12}",
        "model", "LSB", "MSB", "Boundary"
    );
    let tree8 = run(CtxPrefixModel8::new(Counter4::new()), inner.clone(), &buf);
    print_row("tree 8", tree8);
    let suffix8 = run(SlidingWindow5::new(), inner.clone(), &buf);
    print_row("suffix 8", suffix8);
    let tree13 = run(CtxPrefixModel13::new(Counter4::new()), inner.clone(), &buf);
    print_row("tree 13", tree13);
    let suffix13 = run(SlidingWindow10::new(), inner.clone(), &buf);
    print_row("suffix 13", suffix13);
    let tree16 = run(CtxPrefixModel16::new(Counter4::new()), inner.clone(), &buf);
    print_row("tree 16", tree16);
    let suffix16 = run(SlidingWindow13::new(), inner.clone(), &buf);
    print_row("suffix 16", suffix16);

    Ok(())
}

fn print_row(name: &str, values: [u64; 3]) {
    println!(
        "{:<12} {:>12} {:>12} {:>12}",
        name, values[0], values[1], values[2]
    );
}

fn run<M: CtxModel + Clone>(model: M, inner: PrefixModel8<u16>, buf: &[u8]) -> [u64; 3] {
    let mut results = [0; 3];
    for (index, ordering) in [
        HistoryBitOrder::LSB,
        HistoryBitOrder::MSB,
        HistoryBitOrder::Boundary,
    ]
    .iter()
    .copied()
    .enumerate()
    {
        let mut model = model.clone();
        let mut ac = ArithmeticCoder::new_coder();
        let mut stats = ACStats::new();
        let mut history = ACHistory::new(inner.clone(), ordering);
        for byte in buf.iter() {
            unroll_for!(bit in byte, {
                _ = ac.encode(bit, model.predict(), &mut stats);
                model.adapt(bit);
                history.update(bit);
                model.set_ctx(history.hash(15));
            });
        }
        results[index] = stats.result();
    }
    results
}

#[derive(Clone)]
struct SlidingWindow5 {
    stats: Vec<Counter4>,
    ctx: usize,
    align: u32,
}
#[derive(Clone)]
struct SlidingWindow10 {
    stats: Vec<Counter4>,
    ctx: usize,
    align: u32,
}
#[derive(Clone)]
struct SlidingWindow13 {
    stats: Vec<Counter4>,
    ctx: usize,
    align: u32,
}

impl SlidingWindow5 {
    pub fn new() -> Self {
        Self {
            stats: vec![Counter4::new(); 1 << 8],
            ctx: 0,
            align: 0,
        }
    }
}
impl SlidingWindow10 {
    pub fn new() -> Self {
        Self {
            stats: vec![Counter4::new(); 1 << 13],
            ctx: 0,
            align: 0,
        }
    }
}
impl SlidingWindow13 {
    pub fn new() -> Self {
        Self {
            stats: vec![Counter4::new(); 1 << 16],
            ctx: 0,
            align: 0,
        }
    }
}

impl CtxModel for SlidingWindow5 {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn set_ctx(&mut self, hash: u32) {
        self.align = (self.align + 1) % 8;
        // let mask = 0x1f;
        let mask = 0x1f;
        self.ctx = usize!((self.align << 5) | (hash & mask));
    }
}
impl CtxModel for SlidingWindow10 {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn set_ctx(&mut self, hash: u32) {
        self.align = (self.align + 1) % 8;
        self.ctx = usize!((self.align << 10) | (hash & 0x3ff));
    }
}
impl CtxModel for SlidingWindow13 {
    fn predict(&self) -> u16 {
        self.stats[self.ctx].p()
    }

    fn adapt(&mut self, bit: u8) {
        self.stats[self.ctx].update(bit);
    }

    fn set_ctx(&mut self, hash: u32) {
        self.align = (self.align + 1) % 8;
        self.ctx = usize!((self.align << 13) | (hash & 0x1fff));
    }
}
