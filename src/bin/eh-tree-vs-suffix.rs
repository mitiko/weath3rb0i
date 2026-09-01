use std::{fs, io};
use weath3rb0i::{
    entropy_coding::ArithmeticCoder,
    helpers::ACStats,
    history::{ACHistory, History},
    models::{
        counters::*, AdaptiveModel, CtxModel, CtxPrefixModel13, CtxPrefixModel16, CtxPrefixModel8,
        FreezeModel, PrefixModel8,
    },
    unroll_for, usize,
};

fn main() -> io::Result<()> {
    let buf = fs::read("/Users/mitiko/_data/calgary/book1")?;

    let mut model = PrefixModel8::new(Counter4::new());
    model.train(&buf);
    let inner = model.freeze();

    let enc_tree = run(CtxPrefixModel8::new(Counter4::new()), inner.clone(), &buf);
    let enc_suffix = run(SlidingWindow5::new(), inner.clone(), &buf);

    println!("8 tree:   {enc_tree}");
    println!("8 suffix: {enc_suffix}");

    let enc_tree = run(CtxPrefixModel13::new(Counter4::new()), inner.clone(), &buf);
    let enc_suffix = run(SlidingWindow10::new(), inner.clone(), &buf);

    println!("13 tree:   {enc_tree}");
    println!("13 suffix: {enc_suffix}");

    let enc_tree = run(CtxPrefixModel16::new(Counter4::new()), inner.clone(), &buf);
    let enc_suffix = run(SlidingWindow13::new(), inner.clone(), &buf);

    println!("16 tree:   {enc_tree}");
    println!("16 suffix: {enc_suffix}");

    Ok(())
}

fn run(mut model: impl CtxModel, inner: PrefixModel8<u16>, buf: &[u8]) -> u64 {
    let mut ac = ArithmeticCoder::new_coder();
    let mut stats = ACStats::new();
    let mut history = ACHistory::new(15, inner);
    for byte in buf.iter() {
        unroll_for!(bit in byte, {
            _ = ac.encode(bit, model.predict(), &mut stats);
            model.adapt(bit);
            history.update(bit);
            model.set_ctx(history.hash());
        });
    }
    stats.result()
}

struct SlidingWindow5 {
    stats: Vec<Counter4>,
    ctx: usize,
    align: u32,
}
struct SlidingWindow10 {
    stats: Vec<Counter4>,
    ctx: usize,
    align: u32,
}
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
