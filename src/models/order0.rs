use super::{counter::Counter, nib_tree::NibTree, Model, Model4};

pub struct Order0 {
    stats: [[Counter; 15]; 512],
    nt: NibTree,
    ctx: u16,
}

impl Order0 {
    pub fn new() -> Self {
        Self {
            stats: [[Counter::new(); 15]; 512], nt: NibTree::new(), ctx: 0
        }
    }
}

const MASK5: u16 = 15 << 5;  // takes 4 bits and sets last 5 to 0

impl Model4 for Order0 {
    fn predict4(&self, nib: u8) -> [u16; 4] {
        let ctx = usize::from(self.ctx);
        self.nt.get4(nib).map(|idx| self.stats[ctx][idx].p())
    }

    fn update4(&mut self, nib: u8) {
        let ctx = usize::from(self.ctx);
        self.nt.get4(nib).into_iter()
            .zip([nib >> 3, (nib >> 2) & 1, (nib >> 1) & 1, nib & 1])
            .for_each(|(idx, bit)| self.stats[ctx][idx].update(bit));

        let vbit = (self.ctx & 1) ^ 1;
        self.ctx = ((self.ctx << 4) & MASK5) | u16::from(nib << 1) | vbit;
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

        // if it's the last bit of the nibble, we need to use the bit cache..
        if self.nt.bit_id == 3 {
            let nib = (self.nt.cache << 1) | bit;
            let vbit = (self.ctx & 1) ^ 1;
            self.ctx = ((self.ctx << 4) & MASK5) | u16::from(nib << 1) | vbit;
        }
        self.nt.update(bit);
    }
}
