use crate::entropy_coding;
use std::{
    fs::File,
    io::{self, BufReader, Read, Result},
    ops::{Index, IndexMut},
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

impl entropy_coding::arithmetic_coder::ACWrite for ACStats {
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

pub struct RotatingBuffer<T, const N: usize> {
    buf: [T; N],
    pos: usize,
}

impl<T, const N: usize> Index<usize> for RotatingBuffer<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[(self.pos + N - (index + 1)) % N]
    }
}

impl<T, const N: usize> IndexMut<usize> for RotatingBuffer<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buf[(self.pos + N - (index + 1)) % N]
    }
}

impl<T, const N: usize> RotatingBuffer<T, N> {
    pub fn push(&mut self, value: T) {
        self.buf[self.pos] = value;
        self.pos = (self.pos + 1) % N;
    }
}

impl<T: Default + Copy, const N: usize> RotatingBuffer<T, N> {
    pub fn new() -> Self {
        Self { buf: [T::default(); N], pos: 0 }
    }

    pub fn init(value: T) -> Self {
        Self { buf: [value; N], pos: 0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rotating_buffer() {
        let mut rb = RotatingBuffer::<u16, 4>::new();
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4);
        assert_eq!(rb[0], 4);
        assert_eq!(rb[1], 3);
        assert_eq!(rb[2], 2);
        assert_eq!(rb[3], 1);

        rb.push(5);
        assert_eq!(rb[0], 5);
        assert_eq!(rb[1], 4);
        assert_eq!(rb[2], 3);
        assert_eq!(rb[3], 2);

        rb.push(6);
        rb.push(7);
        rb.push(8);
        assert_eq!(rb[0], 8);
        assert_eq!(rb[1], 7);
        assert_eq!(rb[2], 6);
        assert_eq!(rb[3], 5);
    }

    #[test]
    fn rotating_buffer_assingment() {
        let mut rb = RotatingBuffer::<u16, 4>::new();
        rb.push(10);
        rb.push(20);
        rb.push(30);
        rb.push(40);
        assert_eq!(rb[0], 40);
        assert_eq!(rb[1], 30);
        assert_eq!(rb[2], 20);
        assert_eq!(rb[3], 10);

        rb[1] = 99;
        assert_eq!(rb[0], 40);
        assert_eq!(rb[1], 99);
        assert_eq!(rb[2], 20);
        assert_eq!(rb[3], 10);

        rb[0] = 100;
        rb[2] = 98;
        rb[3] = 97;
        assert_eq!(rb[0], 100);
        assert_eq!(rb[1], 99);
        assert_eq!(rb[2], 98);
        assert_eq!(rb[3], 97);
    }
}
