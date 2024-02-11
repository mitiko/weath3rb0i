use std::{
    fs::File,
    io::{BufWriter, Result, Write},
    time::Instant,
};
use weath3rb0i::{
    decode8, encode8,
    entropy_coding::{
        ac_io::{ACReader, ACWriter},
        ArithmeticCoder,
    },
    helpers::cmp,
    models::Model,
};

mod model;
use model::PMHash;

const MAGIC_STR: &[u8; 4] = b"w80i";

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

fn compress(in_file: &str, out_file: &str) -> Result<()> {
    let buf = std::fs::read(in_file)?;
    let mut ac = ArithmeticCoder::new_coder();
    let mut model = PMHash::build(&buf);

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
            &buf[..MAGIC_STR.len()],
            MAGIC_STR,
            "Magic string doesn't match. Check version."
        );
        buf[MAGIC_STR.len()..MAGIC_STR.len() + 8]
            .try_into()
            .map(u64::from_be_bytes)
            .map(usize::try_from)
            .unwrap()
            .unwrap()
    };
    let buf = &buf[MAGIC_STR.len() + 8..];
    let mut out = Vec::with_capacity(len * 4 / 3);
    let mut model = PMHash::init(&buf[..256]);
    let mut reader = ACReader::new(&buf[256..]);
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
