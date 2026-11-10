//! A very small random number generator.
//!
//! We write our own so the project needs no outside crates.
//! It uses the "xorshift" method: scramble a number with a few shifts and XORs.
//! It is NOT good enough for cryptography, but it is plenty for a word game.

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng {
    state: u64, // the current "scrambled" number (must never be 0)
}

impl Rng {
    /// Make a generator that starts from the computer's clock,
    /// so every run of the game is different.
    pub fn from_clock() -> Rng {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap() // the clock is always after 1970, so this cannot fail
            .as_nanos() as u64;
        Rng::from_seed(nanos)
    }

    /// Make a generator from a fixed number.
    /// The same seed always gives the same random numbers - handy for tests!
    pub fn from_seed(seed: u64) -> Rng {
        // xorshift gets stuck if the state is 0, so avoid that.
        let start = if seed == 0 { 88172645463325252 } else { seed };
        let mut rng = Rng { state: start };
        // Throw away a few numbers so that similar seeds do not look similar.
        for _ in 0..20 {
            rng.next_number();
        }
        rng
    }

    /// Give back the next big random number.
    pub fn next_number(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Give back a random number from 0 up to (but not including) `limit`.
    /// For example `below(6)` gives 0, 1, 2, 3, 4 or 5.
    pub fn below(&mut self, limit: usize) -> usize {
        (self.next_number() % limit as u64) as usize
    }

    /// Mix up the order of a list of letters (the "Fisher-Yates shuffle").
    pub fn shuffle(&mut self, letters: &mut [char]) {
        // Walk from the back; swap each spot with a random spot at or before it.
        for i in (1..letters.len()).rev() {
            let j = self.below(i + 1);
            letters.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_same_numbers() {
        let mut a = Rng::from_seed(42);
        let mut b = Rng::from_seed(42);
        for _ in 0..10 {
            assert_eq!(a.next_number(), b.next_number());
        }
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::from_seed(7);
        for _ in 0..1000 {
            assert!(rng.below(6) < 6);
        }
    }

    #[test]
    fn shuffle_keeps_the_same_letters() {
        let mut rng = Rng::from_seed(3);
        let mut letters = vec!['a', 'b', 'c', 'd', 'e'];
        rng.shuffle(&mut letters);
        letters.sort();
        assert_eq!(letters, vec!['a', 'b', 'c', 'd', 'e']);
    }
}
