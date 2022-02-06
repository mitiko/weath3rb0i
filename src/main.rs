// (c) 2022 Dimitar Rusev <mitikodev@gmail.com> licensed under GPL-3.0

use std::fs::{self, File};
use std::io::{BufWriter, Write};

mod hashmap;
mod state_table;
mod range_coder;

use range_coder::RangeCoder;
use state_table::StateTable;
use hashmap::HashMap;


fn main() {
    let timer = std::time::Instant::now();
    let file = "/data/calgary/book1";
    let mut writer = BufWriter::new(File::create("book1.bin").unwrap());
    println!("Compressing {file}");

    let buf = fs::read(file).unwrap();
    let mut map = HashMap::new(1 << 29); // 512 MB
    let state_table = StateTable::new();

    let mut ctx = 0;
    let mut coder = RangeCoder::new();
    writer.write_all(&buf.len().to_be_bytes()).unwrap();

    for byte in buf {
        for nib in [byte >> 4, byte % 15] {
            let hash = ctx as u64;

            let mut slot = map.get_slot(hash);
            let mut states = slot.get_nib(nib);

            coder.encode4(&mut writer, nib, state_table.p_nib(&states));
            state_table.next_nib(&mut states, nib);

            slot.set_nib(nib, states);
            ctx <<= 4; ctx += nib as u32;
        }
    }

    coder.flush(&mut writer);

    println!("Took: {:?}", timer.elapsed());
}
