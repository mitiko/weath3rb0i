// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

use std::fs::File;
use std::io::{BufWriter, Write, BufReader, Read};
use std::path::Path;

use crate::models::*;
use crate::range_coder::RangeCoder;

// TODO: Seperate modeling, prediction and coding
// TODO: Only write modeling data to a file, when in debug mode? - or add a flag to output the model
// TODO: Use ANS as the primary entropy coder
pub fn encode(input_path: &Path, output_path: &Path) {
    let size: u64 = input_path.metadata().unwrap().len();
    let     reader = BufReader::new(File::open(input_path).unwrap());
    let mut writer = BufWriter::new(File::create(output_path).unwrap());

    let mut model0 = Order0::init();
    let mut coder = RangeCoder::new();

    writer.write_all(&size.to_be_bytes()).unwrap();
    for byte_res in reader.bytes() {
        let byte = byte_res.unwrap();
        for nib in [byte >> 4, byte & 15] {
            let p = model0.predict4(nib);
            coder.encode4(&mut writer, nib, p);
            model0.update4(nib);
        }
    }

    coder.flush(&mut writer);
}

pub fn decode(input_path: &Path, output_path: &Path) {
    let mut reader = BufReader::new(File::open(input_path).unwrap());
    let mut writer = BufWriter::new(File::create(output_path).unwrap());

    let mut model0 = Order0::init();
    let mut coder = RangeCoder::new();

    let mut buf = [0; 256];
    reader.read_exact(&mut buf[..8]).unwrap();
    let size = u64::from_be_bytes(buf[..8].try_into().unwrap());
    reader.read_exact(&mut buf[..4]).unwrap();
    coder.init_decode(u32::from_be_bytes(buf[..4].try_into().unwrap()));
    let mut written = 0;

    loop {
        let mut byte = 1;
        let mut eof = false;
        while byte < 256 {
            let p = model0.predict();
            let bit = coder.decode(p);
            model0.update(bit);
            eof = coder.renorm_dec(&mut reader);
            byte = (byte * 2) + bit as usize;
        }
        byte -= 256;
        written += writer.write(&[byte as u8]).unwrap() as u64;
        if written == size || eof { break; }
    }
    writer.flush().unwrap();
}
