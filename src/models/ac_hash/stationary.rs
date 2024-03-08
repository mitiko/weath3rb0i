use crate::{
    models::{ACHashModel, Counter},
    unroll_for,
};

// encodes bits in reverse
#[derive(Clone)]
pub struct StationaryModel {
    table: [u16; 8],
    alignment: u8,
}

impl StationaryModel {
    pub fn new(buf: &[u8]) -> Self {
        let mut model = [Counter::new(); 8];
        for byte in buf {
            let mut i = 7;
            unroll_for!(bit in byte, {
                i = (i + 1) & 7;
                model[i].update(bit);
            });
        }
        let table = [
            model[0].p(),
            model[1].p(),
            model[2].p(),
            model[3].p(),
            model[4].p(),
            model[5].p(),
            model[6].p(),
            model[7].p(),
        ];
        Self { alignment: 0, table }
    }

    pub fn from_table(table: [u16; 8]) -> Self {
        Self { alignment: 0, table }
    }

    pub fn for_book1() -> Self {
        Self::from_table([1, 50188, 62497, 15819, 22545, 31499, 22988, 29616])
    }

    pub fn for_enwik7() -> Self {
        Self::from_table([752, 50314, 58928, 21421, 24680, 30788, 24297, 32530])
    }
}

impl ACHashModel for StationaryModel {
    fn align(&mut self, alignment: u8) {
        self.alignment = alignment;
    }

    fn predict(&mut self, _bit: u8) -> u16 {
        self.alignment = (self.alignment + 7) & 7; // -1 = 7 (mod 8)
        self.table[usize::from(self.alignment)]
    }
}
