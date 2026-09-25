const MULTIPLIER: u64 = 6_364_136_223_846_793_005;

#[derive(Clone, Debug, PartialEq)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut rng = Pcg32 {
            state: 0,
            inc: (stream << 1) | 1,
        };
        rng.advance();
        rng.state = rng.state.wrapping_add(seed);
        rng.advance();
        rng
    }

    fn advance(&mut self) {
        self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(self.inc);
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.advance();
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    pub fn next_f64(&mut self) -> f64 {
        let high = f64::from(self.next_u32() >> 5);
        let low = f64::from(self.next_u32() >> 6);
        (high * 67_108_864.0 + low) / 9_007_199_254_740_992.0
    }

    pub fn below(&mut self, n: u32) -> u32 {
        let n = n.max(1);
        let threshold = n.wrapping_neg() % n;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % n;
            }
        }
    }

    pub fn state_words(&self) -> (u64, u64) {
        (self.state, self.inc)
    }
}
