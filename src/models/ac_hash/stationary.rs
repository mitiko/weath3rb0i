use crate::{
    models::{Counter, Counter4, Model},
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
        let mut model = [Counter4::new(); 8];
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

impl Model for StationaryModel {
    fn predict(&self) -> u16 {
        self.table[usize::from(self.alignment)]
    }

    fn update(&mut self, _bit: u8) {
        self.alignment = (self.alignment + 1) % 8;
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_stationary_model() {
        // repeating d for 2**16 bytes, then a 0
        let d = 0b0000_1000;
        let i = u16::MAX as usize - 1;
        let buf = std::iter::repeat(d)
            .take(i)
            .chain(std::iter::once(0))
            .collect::<Vec<u8>>();

        let mut model = StationaryModel::new(&buf);
        assert_eq!(model.predict(), 1);
        model.update(1);
        assert_eq!(model.predict(), 1);
        model.update(1);
        assert_eq!(model.predict(), 1);
        model.update(1);
        assert_eq!(model.predict(), 1);
        model.update(0);
        assert_eq!(model.predict(), u16::MAX - 1);
    }
}
