// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

pub struct HashMap {
    arr: Vec<Cell>,
    log_cell_count: u8
}

impl HashMap {
    pub fn new(size: usize) -> Self {
        let cell_size = std::mem::size_of::<Cell>();
        let log_cell_count = ((size as f64).log2() - (cell_size as f64).log2()) as u8;
        let cell_count = 1 << log_cell_count;
        println!("Allocating a hashmap of size {} B ({} MB). Cell count is {} (1 << {})", size, size >> 20, cell_count, log_cell_count);
        Self { arr: vec![Cell::empty(); cell_count], log_cell_count }
    }

    // Uses high bits of hash first
    pub fn get_slot(&mut self, hash: u64) -> Slot {
        let index = hash >> (64 - self.log_cell_count);
        self.arr[index as usize].get_slot(hash)
    }
}

#[derive(Clone)]
pub struct Cell {
    hashes: [u8; 6],
    slots:  [u8; 90]
}

impl Cell {
    fn empty() -> Self { Self { hashes: [0; 6], slots: [0; 90] } }

    pub fn get_slot(&mut self, hash: u64) -> Slot<'_> {
        let hashes_concat = u64::from_be_bytes([0, 0, self.hashes[0], self.hashes[1], self.hashes[2], self.hashes[3], self.hashes[4], self.hashes[5]]);
        let mask = (1 << 12) - 1;
        let h = hash & mask;

        let index = if h == hashes_concat & mask      { 0 }
            else if h == (hashes_concat >> 12) & mask { 1 }
            else if h == (hashes_concat >> 24) & mask { 2 }
            else if h == (hashes_concat >> 36) & mask { 3 }
            else {
                // TODO: Select min
                0
            };

        return Slot { index, cell: self };
    }
}

pub struct Slot<'a> {
    index: usize,
    cell: &'a mut Cell
}

impl<'a> Slot<'a> {
    pub fn get_nth_bit(&self, bit_id: usize, nib_ctx: usize) -> u16 {
        let offset = 6 * ((7 >> (3 - bit_id)) + nib_ctx);
        let idx = (3 * self.index) << 1;
        let state = u16::from_be_bytes([self.cell.slots[offset + idx], self.cell.slots[offset + idx + 1]]);

        // TODO: check this becomes cmov
        let mask = (1 << 12) - 1;
        if self.index & 1 == 0 { state >> 4 }
        else                   { state & mask }
    }

    pub fn set_nth_bit(&mut self, bit_id: usize, nib_ctx: usize, new_state: u16) {
        // TODO: optim
        let offset = 6 * ((7 >> (3 - bit_id)) + nib_ctx);
        let idx = (3 * self.index) << 1;

        // TODO: check this becomes cmov
        if self.index & 1 == 0 {
            let bytes = (new_state << 4).to_be_bytes();
            self.cell.slots[offset+idx]   = bytes[0];
            self.cell.slots[offset+idx+1] = bytes[1] | (self.cell.slots[offset+idx+1] & 15)
        }
        else {
            let bytes = new_state.to_be_bytes();
            self.cell.slots[offset+idx]   = bytes[0] | (self.cell.slots[offset+idx] & (15 << 4));
            self.cell.slots[offset+idx+1] = bytes[1];
        }
    }

    pub fn set_nib(&mut self, nib: u8, new_states: [u16; 4]) {
        self.set_nth_bit(0, 0, new_states[0]);
        self.set_nth_bit(1, (nib >> 3) as usize, new_states[1]);
        self.set_nth_bit(2, (nib >> 2) as usize, new_states[2]);
        self.set_nth_bit(3, (nib >> 1) as usize, new_states[3]);
    }

    pub fn get_nib(&self, nib: u8) -> [u16; 4] {
        [
            self.get_nth_bit(0, 0),
            self.get_nth_bit(1, (nib >> 3) as usize),
            self.get_nth_bit(2, (nib >> 2) as usize),
            self.get_nth_bit(3, (nib >> 1) as usize)
        ]
    }
}
