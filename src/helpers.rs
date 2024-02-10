use std::{
    fs::File,
    io::{BufReader, Read, Result},
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

#[macro_export]
macro_rules! encode8 {
    ($byte: expr, $b:ident, $x: block) => {
        let mut $b = $byte >> 7;
        $x;
        $b = ($byte >> 6) & 1;
        $x;
        $b = ($byte >> 5) & 1;
        $x;
        $b = ($byte >> 4) & 1;
        $x;
        $b = ($byte >> 3) & 1;
        $x;
        $b = ($byte >> 2) & 1;
        $x;
        $b = ($byte >> 1) & 1;
        $x;
        $b = $byte & 1;
        $x;
    };
}

#[macro_export]
macro_rules! decode8 {
    ($byte:ident, $bit:ident, $x: block) => {
        let mut $byte = 0;
        let mut $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
        $x;
        $byte = ($byte << 1) | $bit;
    };
}
