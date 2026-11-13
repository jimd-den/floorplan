//! A small seeded RNG built on splitmix64. std only, deterministic:
//! the same u32 seed always yields the same sequence.

/// Splitmix64 generator: one u64 of state, no external crates.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Seeds from a u32. The seed is decorrelated once so small seeds
    /// do not produce visibly similar streams.
    pub fn new(seed: u32) -> Self {
        let mut rng = Self { state: seed as u64 };
        rng.next_u64();
        rng
    }

    /// Next raw u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Next raw u32 (high half of the next u64).
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform f64 in [lo, hi).
    pub fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.unit() * (hi - lo)
    }

    /// Uniform i64 in [lo, hi).
    pub fn range_i64(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_u64() % (hi - lo) as u64) as i64
    }

    /// Uniform usize in [0, n).
    pub fn range_usize(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// Picks one element uniformly.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.range_usize(items.len())]
    }

    /// True with probability p (0.0..=1.0).
    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    /// Fisher-Yates shuffle, in place.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.range_usize(i + 1));
        }
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_the_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_give_different_sequences() {
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
    }

    #[test]
    fn range_f64_stays_inside_its_bounds() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            let v = rng.range_f64(2.0, 5.0);
            assert!((2.0..5.0).contains(&v), "out of bounds: {v}");
        }
    }

    #[test]
    fn range_f64_actually_varies() {
        let mut rng = Rng::new(7);
        let first = rng.range_f64(0.0, 1.0);
        let varied = (0..100).any(|_| rng.range_f64(0.0, 1.0) != first);
        assert!(varied, "range_f64 never varied");
    }

    #[test]
    fn shuffle_keeps_every_element() {
        let mut rng = Rng::new(3);
        let mut items = vec![1, 2, 3, 4, 5, 6, 7, 8];
        rng.shuffle(&mut items);
        let mut sorted = items.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert_ne!(items, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
