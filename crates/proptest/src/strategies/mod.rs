use std::ops::{Deref, DerefMut};

use rand::{CryptoRng, RngCore};

use crate::{Arbitrary, arbitrary};

pub const MAX_STRATEGY_ATTEMPTS: usize = 64;

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
    recursion_limit: usize,
}

impl<R: RngCore + CryptoRng> Generator<R> {
    pub fn build(rng: R) -> Self {
        Self::build_with_limit(rng, usize::MAX)
    }

    pub fn build_with_limit(rng: R, recursion_limit: usize) -> Self {
        Self {
            rng,
            iteration: 0,
            depth: 0,
            recursion_limit,
        }
    }

    pub fn iteration(&self) -> usize {
        self.iteration
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn accept<T>(&mut self, value: T) -> Generation<T> {
        let generation = Generation::Accepted {
            iteration: self.iteration,
            depth: self.depth,
            value,
        };
        self.iteration += 1;
        generation
    }

    pub fn reject<T>(&mut self, value: T) -> Generation<T> {
        let generation = Generation::Rejected {
            iteration: self.iteration,
            depth: self.depth,
            value,
        };
        self.iteration += 1;
        generation
    }

    pub fn recurse<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Generator<R>) -> T,
    {
        let mut guard = DepthGuard::new(self);
        f(&mut guard)
    }
}

struct DepthGuard<'a, R: RngCore + CryptoRng> {
    generator: &'a mut Generator<R>,
}

impl<'a, R: RngCore + CryptoRng> DepthGuard<'a, R> {
    fn new(generator: &'a mut Generator<R>) -> Self {
        if generator.depth >= generator.recursion_limit {
            panic!(
                "#[proptest] strategy recursion exceeded limit of {}",
                generator.recursion_limit,
            );
        }
        generator.depth += 1;
        Self { generator }
    }
}

impl<'a, R: RngCore + CryptoRng> Drop for DepthGuard<'a, R> {
    fn drop(&mut self) {
        self.generator.depth -= 1;
    }
}

impl<'a, R: RngCore + CryptoRng> Deref for DepthGuard<'a, R> {
    type Target = Generator<R>;

    fn deref(&self) -> &Self::Target {
        self.generator
    }
}

impl<'a, R: RngCore + CryptoRng> DerefMut for DepthGuard<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.generator
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
        let mut generator =
            Generator::build_with_limit(ThreadRng::default(), usize::MAX);

        match different::<u8, _>(&mut generator) {
            Generation::Accepted { value: (a, b), .. } => assert_ne!(a, b),
            Generation::Rejected { value: (a, b), .. } => assert_eq!(a, b),
        }
    }

    #[test]
    fn test_vec_not_empty() {
        let mut generator =
            Generator::build_with_limit(ThreadRng::default(), usize::MAX);

        let items: Vec<u8> = vec::not_empty(&mut generator).take();

        assert_ne!(items.len(), 0);
    }

    #[test]
    fn test_recurse_tracks_depth() {
        let mut generator =
            Generator::build_with_limit(ThreadRng::default(), usize::MAX);
        assert_eq!(generator.depth(), 0);

        let result: usize = generator.recurse(|outer| {
            assert_eq!(outer.depth(), 1);
            outer.recurse(|inner| {
                assert_eq!(inner.depth(), 2);
                42
            })
        });

        assert_eq!(result, 42);
        assert_eq!(generator.depth(), 0);
    }
}
