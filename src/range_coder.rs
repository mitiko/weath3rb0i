use std::io::{Read, Write, BufReader, BufWriter};
use std::fs::File;

pub struct RangeCoder {
    low: u32,
    high: u32,
    _x: u32
}

impl RangeCoder {
    pub fn new() -> Self { Self { low: 0, high: 0xffff_ffff, _x: 0 } }
    pub fn _init_decode(&mut self, x: u32) { self._x = x; }

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

    pub fn _decode(&mut self, prob: u32) -> usize {
        let mid = self.low + ((self.high - self.low) >> 12) * prob;
        if self._x <= mid { self.high = mid;     return 1; }
        else             { self.low  = mid + 1; return 0; }
    }

    pub fn renorm_enc(&mut self, stream: &mut BufWriter<File>) {
        while (self.high ^ self.low) & 0xff00_0000 == 0 {
            stream.write(&[(self.high >> 24) as u8]).unwrap();
            self.low <<= 8;
            self.high = (self.high << 8) + 255;
        }
    }

    pub fn _renorm_dec(&mut self, stream: &mut BufReader<File>) -> bool {
        let mut eof = false;

        while (self.high ^ self.low) & 0xff00_0000 == 0 {
            self.low <<= 8;
            self.high = (self.high << 8) + 255;
            let mut byte = [0; 1]; eof = stream.read(&mut byte).unwrap() == 0;
            self._x = (self._x << 8) + byte[0] as u32;
        }

        return eof;
    }

    pub fn flush(&mut self, stream: &mut BufWriter<File>) {
        self.renorm_enc(stream);
        stream.write(&[(self.high >> 24) as u8]).unwrap();
        stream.flush().unwrap();
    }
}