use std::time::Instant;

use weath3rb0i::{
    entropy_coding::package_merge::{canonical, package_merge},
    helpers::histogram,
    models::{Counter, Model},
};

pub struct PMHash {
    table: Vec<(u16, u8)>,
    stats: [Counter; 1 << 8],
    ctx: u8,
}

impl PMHash {
    pub fn build(buf: &[u8]) -> Self {
        let timer = Instant::now();
        let counts = histogram(&buf);
        let code_lens = package_merge(&counts, 12);
        let instance = Self::init(&code_lens);
        println!("[pm] build took {:?}", timer.elapsed());
        instance
    }

    pub fn init(buf: &[u8]) -> Self {
        assert!(buf.len() >= 256);
        let timer = Instant::now();
        let codes = canonical(&buf[..256]);
        println!("[pm] init took {:?}", timer.elapsed());
        Self {
            table: codes,
            stats: [Counter::new(); 1 << 8],
            ctx: 0,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.table.iter().map(|x| x.1).collect()
    }
}

impl Model for PMHash {
    fn predict(&self) -> u16 {
        self.stats[usize::from(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[usize::from(self.ctx)].update(bit);
        self.ctx = (self.ctx << 1) | bit;
    }
}
