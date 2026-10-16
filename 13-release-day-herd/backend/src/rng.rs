#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 { 0 } else { self.next_u64() % bound }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_replays_the_same_launch() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        let left: Vec<u64> = (0..100).map(|_| a.next_u64()).collect();
        let right: Vec<u64> = (0..100).map(|_| b.next_u64()).collect();
        assert_eq!(left, right);
    }

    #[test]
    fn below_stays_inside_the_bound() {
        let mut r = Rng::new(1);
        assert!((0..10_000).all(|_| r.below(17) < 17));
        assert_eq!(r.below(0), 0);
    }
}
