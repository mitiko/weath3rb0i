use std::io::{Read, Write, BufReader, BufWriter};
use std::fs::File;

pub struct RangeCoder {
    low: u32,
    high: u32,
    x: u32,
    byte_buf: [u8; 1]
}

impl RangeCoder {
    pub fn new() -> Self { Self { low: 0, high: 0xffff_ffff, x: 0, byte_buf: [0; 1] } }
    pub fn init_decode(&mut self, x: u32) { self.x = x; }

    pub fn encode(&mut self, bit: u8, prob: u16) {
        let mid = self.low + ((self.high - self.low) >> 12) * prob as u32;
        if bit == 1 { self.high = mid;     }
        else        { self.low  = mid + 1; }
    }

    pub fn encode4(&mut self, stream: &mut BufWriter<File>, nib: u8, probs: [u16; 4]) {
        self.encode(nib >> 3, probs[0]);
        self.renorm_enc(stream);
        self.encode((nib >> 2) & 1, probs[1]);
        self.renorm_enc(stream);
        self.encode((nib >> 1) & 1, probs[2]);
        self.renorm_enc(stream);
        self.encode(nib & 1, probs[3]);
        self.renorm_enc(stream);
    }

    #[allow(clippy::needless_return)] /* For execution path clairity */
    pub fn decode(&mut self, prob: u16) -> u8 {
        let mid = self.low + ((self.high - self.low) >> 12) * prob as u32;
        if self.x <= mid { self.high = mid;     return 1; }
        else             { self.low  = mid + 1; return 0; }
    }

    pub fn renorm_enc(&mut self, stream: &mut BufWriter<File>) {
        while (self.high ^ self.low) & 0xff00_0000 == 0 {
            let byte = (self.high >> 24) as u8;
            let _ = stream.write(&[byte]).expect("Write failed!");

            self.low <<= 8;
            self.high = (self.high << 8) + 255;
        }
    }

    pub fn renorm_dec(&mut self, stream: &mut BufReader<File>) -> bool {
        let mut eof = false;

        while (self.high ^ self.low) & 0xff00_0000 == 0 {
            self.low <<= 8;
            self.high = (self.high << 8) + 255;

            eof = stream.read(&mut self.byte_buf).expect("Read failed!") == 0;
            self.x = (self.x << 8) + self.byte_buf[0] as u32;

            if eof { break; }
        }

        eof
    }

    pub fn flush(&mut self, stream: &mut BufWriter<File>) {
        self.renorm_enc(stream);
        let byte = (self.high >> 24) as u8;
        let _ = stream.write(&[byte]).expect("Write failed!");
        stream.flush().expect("Flush failed!");
    }
}