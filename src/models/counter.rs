#[derive(Copy, Clone)]
pub struct Counter {
    data: [u16; 2]
}

impl Counter {
    pub fn new() -> Self { Self { data: [0; 2] } }

    // TODO: check if not borrowing self (as suggested by clippy) is any beneficial
    pub fn p(&self) -> u16 {
        let c0 = self.data[0] as u64;
        let c1 = self.data[1] as u64;
        let p = (1 << 16) * (c1 + 1) / (c0 + c1 + 2);
        // TODO: Remove 'as' statements, bc they're evil
        p as u16
    }

    pub fn update(&mut self, bit: u8) {
        self.data[bit as usize] += 1;
        if self.data[bit as usize] == u16::MAX {
            self.data[0] >>= 1;
            self.data[1] >>= 1;
        }
    }
}