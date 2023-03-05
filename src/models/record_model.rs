use crate::counters::{ExactRecordKeeper, RecordCounter};
use super::{Model, Model4, nib_tree::NibTree};

pub struct RecordModel {
    stats: [[ExactRecordKeeper; 15]; 512],
    nt: NibTree,
    ctx: u16,
    pos: u16
}

// mask the first nibble and discard last 5 lsb
const MASK: u16 = (1 << 9) - (1 << 5);

impl Model for RecordModel {
    fn new() -> Self {
        Self {
            stats: [[ExactRecordKeeper::new(); 15]; 512],
            nt: NibTree::new(), ctx: 0, pos: 0
        }
    }

    fn predict(&mut self) -> u16 {
        let ctx = usize::from(self.ctx);
        let idx = self.nt.get();
        self.stats[ctx][idx].predict(self.pos)
    }

    fn update(&mut self, bit: u8) {
        let ctx = usize::from(self.ctx);
        let idx = self.nt.get();
        self.stats[ctx][idx].update(self.pos, bit);

        // if it's the last bit of the nibble, we need to use the bit cache..
        if let Some(nib) = self.nt.update(bit) {
            let vbit = (self.ctx & 1) ^ 1;
            self.pos = self.pos.wrapping_add(vbit ^ 1);
            self.ctx = ((self.ctx << 4) & MASK) | u16::from(nib << 1) | vbit;
        }
    }
}

impl Model4 for RecordModel {
    fn predict4(&mut self, nib: u8) -> [u16; 4] {
        let ctx = usize::from(self.ctx);
        self.nt.get4(nib).map(|idx| self.stats[ctx][idx].predict(self.pos))
    }

    fn update4(&mut self, nib: u8) {
        let ctx = usize::from(self.ctx);
        self.nt.get4(nib).into_iter()
            .zip([nib >> 3, (nib >> 2) & 1, (nib >> 1) & 1, nib & 1])
            .for_each(|(idx, bit)| self.stats[ctx][idx].update(self.pos, bit));

        let vbit = (self.ctx & 1) ^ 1;
        self.pos = self.pos.wrapping_add(vbit ^ 1);
        self.ctx = ((self.ctx << 4) & MASK) | u16::from(nib << 1) | vbit;
    }
}
