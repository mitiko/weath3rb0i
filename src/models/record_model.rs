use crate::counters::{ExactRecordKeeper, RecordCounter};
use super::{Model, Model4, nib_tree::NibTree};

pub struct RecordModel<TRecordCounter> {
    stats: [[TRecordCounter; 15]; 512],
    nt: NibTree,
    ctx: u16,
    pos: u16
}

// mask the first nibble and discard last 5 lsb
const MASK: u16 = (1 << 9) - (1 << 5);

impl<T: RecordCounter> Model for RecordModel<T> {
    fn new() -> Self {
        Self {
            stats: [[T::new(); 15]; 512],
            nt: NibTree::new(), ctx: 0, pos: 0
        }
    }

    fn predict(&self) -> u16 {
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
            self.ctx = ((self.ctx << 4) & MASK) | u16::from(nib << 1) | vbit;
            self.pos = self.pos.wrapping_add(vbit); // adds 1 every 2n+1 nibble
        }
    }
}

impl<T: RecordCounter> Model4 for RecordModel<T> {
    fn predict4(&self, nib: u8) -> [u16; 4] {
        let ctx = usize::from(self.ctx);
        self.nt.get4(nib).map(|idx| self.stats[ctx][idx].predict(self.pos))
    }

    fn update4(&mut self, nib: u8) {
        let ctx = usize::from(self.ctx);
        self.nt.get4(nib).into_iter()
            .zip([nib >> 3, (nib >> 2) & 1, (nib >> 1) & 1, nib & 1])
            .for_each(|(idx, bit)| self.stats[ctx][idx].update(self.pos, bit));

        let vbit = (self.ctx & 1) ^ 1;
        self.ctx = ((self.ctx << 4) & MASK) | u16::from(nib << 1) | vbit;
        self.pos = self.pos.wrapping_add(vbit); // adds 1 every 2n+1 nibble
    }
}

