use crate::{
    entropy_coding::package_merge::{canonical, package_merge},
    helpers::histogram,
    u8,
};

use super::History;

#[derive(Clone)]
pub struct HuffHistory {
    pos: u64,
    bits: u64,
    compressed_bits: u32,
    table: Vec<(u16, u8)>,
    rem_table: Vec<(u16, u8)>,
}

impl HuffHistory {
    pub fn new(buf: &[u8], huff_size: u8, rem_huff_size: u8) -> Self {
        let counts = histogram(&buf);
        let code_lens = package_merge(&counts, huff_size);
        let mut huffman = canonical(&code_lens);
        for i in 0..huffman.len() {
            let (mut code, len) = huffman[i];
            code = code.reverse_bits().overflowing_shr(u32::from(16 - len)).0;
            huffman[i] = (code, len);
        }

        let mut rem_counts = vec![0; 256];
        for (byte, count) in counts.iter().enumerate() {
            for bit_len in 0..8 {
                let sym_bits = byte >> (8 - bit_len);
                let sym = (1 << bit_len) | sym_bits;
                rem_counts[sym] += count;
            }
        }
        let rem_code_lens = package_merge(&rem_counts, rem_huff_size); // TODO: maybe other param?
        let mut rem_huffman = canonical(&rem_code_lens);
        for i in 0..rem_huffman.len() {
            let (mut code, len) = rem_huffman[i];
            code = code.reverse_bits().overflowing_shr(u32::from(16 - len)).0;
            rem_huffman[i] = (code, len);
        }

        Self {
            pos: 0,
            bits: 0,
            compressed_bits: 0,
            table: huffman,
            rem_table: rem_huffman,
        }
    }
}

impl History for HuffHistory {
    fn update(&mut self, bit: u8) {
        self.bits = (self.bits << 1) | u64::from(bit);
        self.pos += 1;
    }

    fn hash(&mut self) -> u32 {
        let alignment = self.pos & 7;
        if alignment == 0 {
            let byte = u8!(self.bits & 255);
            let (code, len) = self.table[usize::from(byte)];
            self.compressed_bits = (self.compressed_bits << len) | u32::from(code);
        }
        let mask = (1 << alignment) - 1;
        let rem_bits = u8!(self.bits & mask);
        let rem_sym = rem_bits | (1 << alignment);
        let (code, len) = self.rem_table[usize::from(rem_sym)];
        (self.compressed_bits << len) | u32::from(code)
    }
}
