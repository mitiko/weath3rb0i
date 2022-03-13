pub struct AlphabetOrderManager<const N: usize> {
    enc_table:      [u16; N],
    dec_table:      [u16; N],
    counts:        [u64; N],
    sorted_counts: [u64; N]
}

// TODO: Encode dec_table to file
// TODO: Use a BufReader instead of a Vec<u8>
impl<const N: usize> AlphabetOrderManager<N> {

    pub fn init() -> Self {
        Self { enc_table: [0; N], dec_table: [0; N], counts: [0; N], sorted_counts: [0; N] }
    }

    pub fn analyze8(&mut self, buf: &[u8]) {
        // Count frequencies
        buf.iter().for_each(|&sym| self.counts[sym as usize] += 1);

        // Sort
        self.sorted_counts.copy_from_slice(&self.counts);
        self.sorted_counts.sort_unstable();

        self.gen_tables();
    }

    pub fn analyze12_16(&mut self, buf: &[u8]) {
        // Count frequencies
        buf.chunks_exact(2)
            .map(|sym| u16::from_be_bytes([sym[0], sym[1]]))
            .for_each(|idx| self.counts[idx as usize] += 1);

        // Sort
        self.sorted_counts.copy_from_slice(&self.counts);
        self.sorted_counts.sort_unstable();

        self.gen_tables();
    }

    pub fn encode8(&self, buf: &mut Vec<u8>) {
        buf.iter_mut()
            .for_each(|sym| *sym = self.enc_table[*sym as usize] as u8);
    }

    pub fn decode8(&self, buf: &mut Vec<u8>) {
        buf.iter_mut()
            .for_each(|sym| *sym = self.dec_table[*sym as usize] as u8);
    }

    pub fn encode12_16(&self, buf: &mut Vec<u8>) {
        buf.chunks_exact_mut(2).for_each(|chunk| {
            let val = u16::from_be_bytes([chunk[0], chunk[1]]);
            let out = self.enc_table[val as usize].to_be_bytes();
            chunk[0] = out[0]; chunk[1] = out[1];
        });
    }

    pub fn decode12_16(&self, buf: &mut Vec<u8>) {
        buf.chunks_exact_mut(2) .for_each(|chunk| {
            let val = u16::from_be_bytes([chunk[0], chunk[1]]);
            let out = self.dec_table[val as usize].to_be_bytes();
            chunk[0] = out[0]; chunk[1] = out[1];
        });
    }

    fn gen_tables(&mut self) {
        // Populate encode and decode tables with linear search (O(n^2) in total)
        let mut idx = 0;
        let mut prev = 0;
        for (sorted_idx, &value) in self.sorted_counts.iter().enumerate() {
            if value != prev { idx = 0; }

            while idx < N {
                if self.counts[idx] == value { break; }
                idx += 1;
            }

            self.enc_table[idx] = sorted_idx as u16;
            self.dec_table[sorted_idx] = idx as u16;
            prev = value;
        }
    }
}
