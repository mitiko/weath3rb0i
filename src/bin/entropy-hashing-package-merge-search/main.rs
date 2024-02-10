use std::{
    fs::File,
    io::{BufWriter, Result, Write},
    time::Instant,
};
use weath3rb0i::{
    decode8, encode8,
    entropy_coding::{
        ac_io::{ACReader, ACWriter},
        package_merge::{canonical, package_merge},
        ArithmeticCoder,
    },
    helpers::{cmp, histogram},
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
        encode8!(byte, bit, {
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
        decode8!(byte, bit, {
            let p = model.predict();
            bit = ac.decode(p, &mut reader)?;
            model.update(bit);
        });
        out.push(byte);
    }
    File::create(out_file)?.write_all(&out)?;
    Ok(())
}
