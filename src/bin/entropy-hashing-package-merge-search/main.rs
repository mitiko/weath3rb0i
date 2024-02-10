use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Result, Write},
    time::Instant,
};
use weath3rb0i::{
    entropy_coding::{
        ac_io::{ACReader, ACWriter},
        package_merge::{canonical, package_merge},
        ArithmeticCoder,
    },
    models::Counter,
};

const MAGIC_STR: &[u8; 8] = b"w3bi00\0\0";

fn main() -> Result<()> {
    let file = "/Users/mitiko/_data/book1";
    let enc_file = "book1.bin";
    let dec_file = "book1.orig";

    let timer = Instant::now();
    compress(file, enc_file)?;

    let in_size = File::open(file)?.metadata()?.len();
    let enc_size = File::open(enc_file)?.metadata()?.len();
    let ratio = enc_size as f64 / in_size as f64;
    println!(
        "Compressed   {in_size} -> {enc_size} ({:.3}) in {:?}",
        ratio,
        timer.elapsed()
    );

    let timer = Instant::now();
    decompress(enc_file, dec_file)?;
    let dec_size = File::open(dec_file)?.metadata()?.len();
    println!(
        "Decompressed {enc_size} -> {dec_size}         in {:?}",
        timer.elapsed()
    );

    cmp(file, "book1.orig")?;
    Ok(())
}

struct Model {
    table: Vec<(u16, u8)>,
    stats: [Counter; 1 << 11],
    ctx: u8,
}

impl Model {
    fn build(buf: &[u8]) -> Self {
        let counts = histogram(&buf);
        let code_lens = package_merge(&counts, 12);
        Self::init(&code_lens)
    }

    fn init(buf: &[u8]) -> Self {
        assert!(buf.len() >= 256);
        Self {
            table: canonical(&buf[..256]),
            stats: [Counter::new(); 1 << 11],
            ctx: 0,
        }
    }

    fn serialize(&self) -> Vec<u8> {
        self.table.iter().map(|x| x.1).collect()
    }

    fn predict(&self) -> u16 {
        self.stats[usize::from(self.ctx)].p()
    }

    fn update(&mut self, bit: u8) {
        self.stats[usize::from(self.ctx)].update(bit);
        self.ctx = (self.ctx << 1) | bit;
    }
}

macro_rules! encode {
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

macro_rules! decode {
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

fn histogram(buf: &[u8]) -> Vec<u32> {
    let mut res = vec![0; 256];
    for &byte in buf {
        res[usize::from(byte)] += 1;
    }
    res
}

fn compress(in_file: &str, out_file: &str) -> Result<()> {
    let buf = std::fs::read(in_file)?;
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = Model::build(&buf);

    let mut writer = {
        let mut writer = BufWriter::new(File::create(out_file)?);
        writer.write_all(MAGIC_STR)?;
        writer.write_all(&u64::try_from(buf.len()).unwrap().to_be_bytes())?;
        writer.write_all(&model.serialize())?;
        ACWriter::new(writer)
    };

    for byte in buf {
        encode!(byte, bit, {
            let p = model.predict();
            model.update(bit);
            ac.encode(bit, p, &mut writer)?;
        });
    }

    // TODO: flush AC on Drop
    ac.flush(&mut writer)?;
    Ok(())
}

fn decompress(in_file: &str, out_file: &str) -> Result<()> {
    let buf = std::fs::read(in_file)?;
    let len = {
        assert_eq!(
            &buf[..8],
            MAGIC_STR,
            "Magic string doesn't match. Check version."
        );
        let x = u64::from_be_bytes(buf[8..16].try_into().unwrap());
        usize::try_from(x).unwrap()
    };
    let mut out = Vec::with_capacity(len * 4 / 3);
    let mut model = Model::init(&buf[16..272]);
    let mut reader = ACReader::new(&buf[272..]);
    let mut ac = ArithmeticCoder::new_decoder(&mut reader)?;

    for _ in 0..len {
        decode!(byte, bit, {
            let p = model.predict();
            bit = ac.decode(p, &mut reader)?;
            model.update(bit);
        });
        out.push(byte);
    }
    File::create(out_file)?.write_all(&out)?;
    Ok(())
}

fn cmp(file1: &str, file2: &str) -> Result<()> {
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
