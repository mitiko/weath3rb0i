use crate::entropy_coding;
use std::{
    fs::File,
    io::{self, BufReader, Read, Result},
};

pub fn cmp(file1: &str, file2: &str) -> Result<()> {
    let f1 = File::open(file1)?;
    let f2 = File::open(file2)?;

    let l1 = f1.metadata().unwrap().len();
    let l2 = f2.metadata().unwrap().len();

    let r1 = BufReader::new(f1);
    let r2 = BufReader::new(f2);

    let mut lines = 0;
    let bytes1 = r1.bytes().map(|b| b.unwrap());
    let bytes2 = r2.bytes().map(|b| b.unwrap());
    for (pos, (b1, b2)) in bytes1.zip(bytes2).enumerate() {
        assert_eq!(b1, b2, "Files differ at byte {}, line {}", pos, lines);
        lines += usize::from(b1 == b'\n');
    }

    assert_eq!(l1, l2, "File 1 is {} bytes and file 2 is {} bytes", l1, l2);
    println!("Compare: OK");
    Ok(())
}

pub fn histogram(buf: &[u8]) -> Vec<u32> {
    let mut res = vec![0; 256];
    for &byte in buf {
        res[usize::from(byte)] += 1;
    }
    res
}

pub fn histogram_simd(buf: &[u8]) -> Vec<u32> {
    let mut res0 = vec![0; 256];
    let mut res1 = vec![0; 256];
    let mut res2 = vec![0; 256];
    let mut res3 = vec![0; 256];
    let iter = buf.chunks_exact(4);

    for &byte in iter.remainder() {
        res0[usize::from(byte)] += 1;
    }
    for chunk in iter {
        res0[usize::from(chunk[0])] += 1;
        res1[usize::from(chunk[1])] += 1;
        res2[usize::from(chunk[2])] += 1;
        res3[usize::from(chunk[3])] += 1;
    }
    for i in 0..256 {
        res0[i] += res1[i] + res2[i] + res3[i];
    }
    res0
}

pub struct ACStats {
    bit_count: u64,
    rev_bits: u64,
}

impl ACStats {
    pub fn new() -> Self {
        Self { bit_count: 0, rev_bits: 0 }
    }

    /// Bytes in compressed size roughly
    pub fn result(&self) -> u64 {
        self.bit_count / 8
    }
}

impl entropy_coding::ACWrite for ACStats {
    fn inc_parity(&mut self) {
        self.rev_bits += 1;
    }

    fn write_bit(&mut self, _bit: impl TryInto<u8>) -> io::Result<()> {
        self.bit_count += 1 + self.rev_bits;
        self.rev_bits = 0;
        Ok(())
    }

    fn flush(&mut self, _padding: u32) -> io::Result<()> {
        Ok(())
    }
}
