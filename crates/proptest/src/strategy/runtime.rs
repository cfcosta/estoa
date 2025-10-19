use std::ops::{Deref, DerefMut};

use rand::{CryptoRng, RngCore, rngs::ThreadRng};

use super::{Strategy, ValueTree};
use crate::arbitrary::Arbitrary;

pub(crate) const MAX_STRATEGY_ATTEMPTS: usize = 64;

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
            Generation::Accepted { value, .. }
            | Generation::Rejected { value, .. } => value,
        }
    }

    pub fn map<U, F>(self, f: F) -> Generation<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Generation::Accepted {
                iteration,
                depth,
                value,
            } => Generation::Accepted {
                iteration,
                depth,
                value: f(value),
            },
            Generation::Rejected {
                iteration,
                depth,
                value,
            } => Generation::Rejected {
                iteration,
                depth,
                value: f(value),
            },
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
        Self {
            rng,
            iteration: 0,
            depth: 0,
            recursion_limit: 10000,
        }
    }

    pub fn with_limit(mut self, recursion_limit: usize) -> Self {
        self.recursion_limit = recursion_limit;
        self
    }

    pub fn iteration(&self) -> usize {
        self.iteration
    }

    pub fn advance_iteration(&mut self) {
        self.iteration = self.iteration.saturating_add(1);
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn accept<T>(&self, value: T) -> Generation<T> {
        Generation::Accepted {
            iteration: self.iteration,
            depth: self.depth,
            value,
        }
    }

    pub fn reject<T>(&self, value: T) -> Generation<T> {
        Generation::Rejected {
            iteration: self.iteration,
            depth: self.depth,
            value,
        }
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

pub type DefaultGenerator = Generator<ThreadRng>;

pub struct IntegratedAdapter<S>
where
    S: Strategy,
    S::Value: Clone,
{
    strategy: S,
}

impl<S> IntegratedAdapter<S>
where
    S: Strategy,
    S::Value: Clone,
{
    pub fn new(strategy: S) -> Self {
        Self { strategy }
    }

    pub fn generate(
        &mut self,
        generator: &mut DefaultGenerator,
    ) -> Generation<S::Value> {
        match self.strategy.new_tree(generator) {
            Generation::Accepted {
                iteration,
                depth,
                value,
            } => Generation::Accepted {
                iteration,
                depth,
                value: value.current().clone(),
            },
            Generation::Rejected {
                iteration,
                depth,
                value,
            } => Generation::Rejected {
                iteration,
                depth,
                value: value.current().clone(),
            },
        }
    }
}

pub fn adapt<S>(strategy: S) -> IntegratedAdapter<S>
where
    S: Strategy,
    S::Value: Clone,
{
    IntegratedAdapter::new(strategy)
}

pub fn execute<S>(
    adapter: &mut IntegratedAdapter<S>,
    generator: &mut DefaultGenerator,
) -> Generation<S::Value>
where
    S: Strategy,
    S::Value: Clone,
{
    adapter.generate(generator)
}

pub fn from_arbitrary<T>(generator: &mut DefaultGenerator) -> Generation<T>
where
    T: Arbitrary,
{
    T::generate(generator)
}

pub struct ConstantValueTree<T> {
    value: T,
}

impl<T> ConstantValueTree<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> ValueTree for ConstantValueTree<T> {
    type Value = T;

    fn current(&self) -> &Self::Value {
        &self.value
    }

    fn simplify(&mut self) -> bool {
        false
    }

    fn complicate(&mut self) -> bool {
        false
    }
}
