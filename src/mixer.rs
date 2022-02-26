// Linear interpolation

pub struct Mixer2 {
    w: f64,
    del_p: f64,
    st_w: f64,
    p: f64
}

// fn sq(x: f64) -> f64 { (x / (1.0 - x)).ln() }
fn st(x: f64) -> f64 { 1.0 / (1.0 + (-x).exp()) }

impl Mixer2 {
    pub fn init() -> Self { Self { w: 0.5, del_p: 0.0, st_w: 0.0, p: 0.5 } }

    pub fn mix(&mut self, p1: u16, p2: u16) -> u16 {
        // Normalize weight
        self.st_w = st(self.w);
        // Normalize inputs
        let x1 = p1 as f64 / (1 << 16) as f64;
        let x2 = p2 as f64 / (1 << 16) as f64;
        // Mix
        self.del_p = x1 - x2;
        self.p = self.del_p * self.st_w + x2;
        // "Denormalize" p
        return ((1 << 16) as f64 * self.p) as u16;
    }

    pub fn update(&mut self, bit: u8) {
        let lr = 0.1; // learning rate
        let c = -std::f64::consts::LOG2_E;
        let dl = c / (if bit == 1 { self.p } else { self.p - 1.0 });
        let dw = dl * self.del_p * self.st_w * (1.0 - self.st_w);

        self.w = self.w - lr * dw;
    }

    pub fn mix4(&mut self, p1: [u16; 4], p2: [u16; 4], nib: u8) -> [u16; 4] {
        let p_0 = self.mix(p1[0], p2[0]); self.update( nib >> 3);
        let p_1 = self.mix(p1[1], p2[1]); self.update((nib >> 2) & 1);
        let p_2 = self.mix(p1[2], p2[2]); self.update((nib >> 1) & 1);
        let p_3 = self.mix(p1[3], p2[3]); self.update( nib       & 1);
        [p_0, p_1, p_2, p_3]
    }
}
