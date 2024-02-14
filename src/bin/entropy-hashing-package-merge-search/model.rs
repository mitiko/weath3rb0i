use std::time::Instant;

use weath3rb0i::{
    entropy_coding::package_merge::{canonical, package_merge, meta_tree},
    helpers::histogram,
    models::{Counter, Model},
};

pub struct PMHash {
    table: Vec<(u16, u8)>, // TODO: reverse the table so we get: code -> symbol
    meta_table: Vec<(u16, u8)>, // TODO: keep track of state
    meta_state: u16,
    stats: Vec<Counter>,
    history: u64,
    raw_history: u64,
    mask: u64,
}

impl PMHash {
    pub fn build(buf: &[u8], bits: u8, tree_depth: u8, meta_tree_depth: u8) -> Self {
        let timer = Instant::now();
        let counts = histogram(&buf);
        let code_lens = package_merge(&counts, tree_depth);
        let codes = canonical(&code_lens);
        // rev_codes(&mut code_lens);

        let meta_counts = meta_tree(&codes, &counts);
        let meta_code_lens = package_merge(&meta_counts, meta_tree_depth);
        let meta_codes = canonical(&meta_code_lens);
        // rev_codes(&mut meta_code_lens);

        println!("[pm] build took {:?}", timer.elapsed());
        Self {
            table: codes,
            meta_table: meta_codes,
            meta_state: 0,
            stats: vec![Counter::new(); 1 << bits],
            mask: (1 << bits) - 1,
            history: 0,
            raw_history: 0,
        }
    }
}

// TODO: check if that helps
pub fn rev_codes(codes: &mut [(u16, u8)]) {
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
        let (code, len) = self.meta_table[usize::from(self.meta_state)];
        let ctx = (self.history << len) | u64::from(code);
        let ctx = usize::try_from(ctx & self.mask).unwrap();
        self.stats[ctx].p()
    }

    fn update(&mut self, bit: u8) {
        let (code, len) = self.meta_table[usize::from(self.meta_state)];
        let ctx = (self.history << len) | u64::from(code);
        let ctx = usize::try_from(ctx & self.mask).unwrap();
        self.stats[ctx].update(bit);

        self.raw_history = (self.raw_history << 1) | u64::from(bit);
        self.meta_state <<= 1;
        self.meta_state |= u16::from(bit);
        let (code, len) = self.meta_table[usize::from(self.meta_state)];
        if code == 1 && len == 1 {
            // decode prev_state or read fixed number of bits (8) from raw bitstream
            let sym = usize::try_from(self.raw_history & 255).unwrap();
            let (code, len) = self.table[sym];
            self.history <<= len;
            self.history |= u64::from(code);
        }
    }
}
