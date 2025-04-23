// use std::time::Instant;

use weath3rb0i::{
    entropy_coding::package_merge::{canonical, meta_tree, package_merge},
    helpers::histogram,
    models::{Counter, Model},
    u16, u32, usize,
};

pub struct PMHash {
    stats: Vec<Counter>,
    history: PMHistory,
}

pub struct PMHistory {
    raw_history: u64,
    history: u64,
    align: u8,
    // TODO: reverse the table so we get: code -> symbol
    table: Vec<(u16, u8)>,
    meta_table: Vec<(u16, u8)>,
    mask: u64,
}

impl PMHistory {
    pub fn hash(&self) -> u32 {
        let isym = (1 << self.align) | ((self.raw_history & 255) >> (8 - self.align));
        let (code, len) = self.meta_table[usize!(isym)];
        let ctx = (self.history << len) | u64::from(code);
        u32!(ctx & self.mask)
    }

    pub fn update(&mut self, bit: u8) {
        self.align = (self.align + 1) & 7;

        if self.align == 0 {
            let (code, len) = self.table[usize!(self.raw_history & 255)];
            self.history <<= len;
            self.history |= u64::from(code);
        }
        self.raw_history = (self.raw_history << 1) | u64::from(bit);
    }
}

impl PMHash {
    pub fn build(buf: &[u8], bits: u8, tree_depth: u8, meta_tree_depth: u8) -> Self {
        // let timer = Instant::now();
        let counts = histogram(&buf);
        let code_lens = package_merge(&counts, tree_depth);
        let mut codes = canonical(&code_lens);
        _rev_codes(&mut codes);

        let meta_counts = meta_tree(&counts);
        let meta_code_lens = package_merge(&meta_counts, meta_tree_depth);
        let mut meta_codes = canonical(&meta_code_lens);
        _rev_codes(&mut meta_codes);

        // println!("[pm] build took {:?}", timer.elapsed());
        Self {
            stats: vec![Counter::new(); 1 << bits],
            history: PMHistory {
                raw_history: 0,
                history: 0,
                align: 0,
                table: codes,
                meta_table: meta_codes,
                mask: (1 << bits) - 1,
            },
        }
    }
}

// TODO: check if that helps
pub fn _rev_codes(codes: &mut [(u16, u8)]) {
    for i in 0..codes.len() {
        let (mut code, len) = codes[i];
        let mut new_code = 0;
        for _ in 0..len {
            new_code |= code & 1;
            new_code <<= 1;
            code >>= 1;
        }
        codes[i] = (new_code, len);
    }
}

impl Model for PMHash {
    fn predict(&self) -> u16 {
        let ctx = usize!(self.history.hash());
        self.stats[ctx].p()
    }

    fn update(&mut self, bit: u8) {
        let ctx = usize!(self.history.hash());
        self.stats[ctx].update(bit);
        self.history.update(bit);
    }
}
