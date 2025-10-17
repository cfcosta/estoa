use std::ops::{Deref, DerefMut};

use rand::{CryptoRng, RngCore};

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
