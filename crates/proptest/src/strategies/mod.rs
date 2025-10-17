use rand::{CryptoRng, RngCore};

use crate::{Arbitrary, arbitrary};

pub enum Generation<T> {
    Accepted {
        iteration: usize,
        depth: usize,
        value: T,
    },
    Rejected {
        iteration: usize,
        depth: usize,
        value: T,
    },
}

impl<T> Generation<T> {
    pub fn take(self) -> T {
        match self {
            Generation::Accepted { value: v, .. } => v,
            Generation::Rejected { value: v, .. } => v,
        }
    }
}

pub struct Generator<R> {
    pub rng: R,
    iteration: usize,
    depth: usize,
}

impl<R: RngCore + CryptoRng> Generator<R> {
    pub fn build(rng: R) -> Self {
        Self {
            rng,
            iteration: 0,
            depth: 0,
        }
    }

    pub fn accept<T>(&mut self, value: T) -> Generation<T> {
        Generation::Accepted {
            iteration: self.iteration,
            depth: self.depth,
            value,
        }
    }

    pub fn reject<T>(&mut self, value: T) -> Generation<T> {
        Generation::Rejected {
            iteration: self.iteration,
            depth: self.depth,
            value,
        }
    }
}

pub fn different<T: Arbitrary + PartialEq, R: RngCore + CryptoRng>(
    generator: &mut Generator<R>,
) -> Generation<(T, T)> {
    let (a, b) = (arbitrary(&mut generator.rng), arbitrary(&mut generator.rng));

    if a != b {
        generator.accept((a, b))
    } else {
        generator.reject((a, b))
    }
}

pub mod vec {
    use rand::{CryptoRng, Rng, RngCore};

    use super::{Generation, Generator};
    use crate::{Arbitrary, arbitrary, arbitrary::COLLECTION_MAX_LEN};

    pub fn not_empty<T: Arbitrary + PartialEq, R: RngCore + CryptoRng>(
        generator: &mut Generator<R>,
    ) -> Generation<Vec<T>> {
        let len = generator.rng.random_range(1..=COLLECTION_MAX_LEN);

        let mut result = vec![];

        for _ in 0..len {
            result.push(arbitrary(&mut generator.rng));
        }

        generator.accept(result)
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::ThreadRng;

    use super::*;

    #[test]
    fn test_not_equal() {
        let mut generator = Generator::build(ThreadRng::default());

        match different::<u8, _>(&mut generator) {
            Generation::Accepted { value: (a, b), .. } => assert_ne!(a, b),
            Generation::Rejected { value: (a, b), .. } => assert_eq!(a, b),
        }
    }

    #[test]
    fn test_vec_not_empty() {
        let mut generator = Generator::build(ThreadRng::default());

        let items: Vec<u8> = vec::not_empty(&mut generator).take();

        assert_ne!(items.len(), 0);
    }
}
