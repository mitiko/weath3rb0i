use super::{counter::Counter, nib_tree::NibTree, Model, Model4};

pub struct Order0 {
    stats: [[Counter; 15]; 512],
    nt: NibTree,
    ctx: u16,
    vbit: u16,
    bits: u16
}

impl Order0 {
    pub fn new() -> Self {
        Self {
            stats: [[Counter::new(); 15]; 512],
            nt: NibTree::new(),
            ctx: 0, vbit: 0, bits: 0
        }
    }
}

const MASK: u16 = (1 << 9) - 1; // takes last 9 bits

impl Model4 for Order0 {
    fn predict4(&self, nib: u8) -> [u16; 4] {
        let ctx = usize::from(self.ctx);
        self.nt.get4(nib).map(|idx| self.stats[ctx][idx].p())
    }

    fn update4(&mut self, nib: u8) {
        let ctx = usize::from(self.ctx);
        let [idx1, idx2, idx3, idx4] = self.nt.get4(nib);
        self.stats[ctx][idx1].update(nib >> 3);
        self.stats[ctx][idx2].update((nib >> 2) & 1);
        self.stats[ctx][idx3].update((nib >> 1) & 1);
        self.stats[ctx][idx4].update(nib & 1);

        self.vbit ^= 1;
        self.bits = (self.bits << 4) | u16::from(nib);
        self.ctx = MASK & (self.bits << 1) | self.vbit;
    }
}

impl Model for Order0 {
    fn predict(&self) -> u16 {
        let ctx = usize::from(self.ctx);
        let idx = self.nt.get();
        self.stats[ctx][idx].p()
    }

    fn update(&mut self, bit: u8) {
        let ctx = usize::from(self.ctx);
        let idx = self.nt.get();
        self.stats[ctx][idx].update(bit);
        self.nt.update(bit);

        self.bits = (self.bits << 1) | u16::from(bit);
        if self.nt.bit_id == 0 {
            self.vbit ^= 1;
            self.ctx = MASK & (self.bits << 1) | self.vbit;
        }
    }
}
