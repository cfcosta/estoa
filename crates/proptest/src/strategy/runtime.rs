use std::ops::{
    Deref,
    DerefMut,
    Range,
    RangeFrom,
    RangeFull,
    RangeInclusive,
    RangeTo,
    RangeToInclusive,
};

use rand::{CryptoRng, Rng, RngCore, rngs::ThreadRng};

use super::{Strategy, ValueTree};
use crate::arbitrary::{Arbitrary, COLLECTION_MAX_LEN};

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

pub fn build_default_generator(recursion_limit: usize) -> DefaultGenerator {
    Generator::build_with_limit(rand::rng(), recursion_limit)
}

pub trait SizeHint {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize;
}

fn clamp_bounds(min: usize, max: Option<usize>) -> (usize, usize) {
    if min > COLLECTION_MAX_LEN {
        panic!(
            "size hint minimum {} exceeds maximum supported length {}",
            min, COLLECTION_MAX_LEN
        );
    }
    let clamped_min = min.min(COLLECTION_MAX_LEN);
    let clamped_max = max
        .map(|m| m.min(COLLECTION_MAX_LEN))
        .unwrap_or(COLLECTION_MAX_LEN);
    if clamped_min > clamped_max {
        panic!(
            "size hint minimum {} exceeds maximum {} after clamping",
            clamped_min, clamped_max
        );
    }
    (clamped_min, clamped_max)
}

impl SizeHint for usize {
    fn pick<R: Rng + ?Sized>(&self, _rng: &mut R) -> usize {
        if *self > COLLECTION_MAX_LEN {
            panic!(
                "size hint {} exceeds maximum supported length {}",
                self, COLLECTION_MAX_LEN
            );
        }
        *self
    }
}

impl SizeHint for Range<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        if self.start >= self.end {
            panic!("size hint range {}..{} is empty", self.start, self.end);
        }
        let (min, max) = clamp_bounds(self.start, Some(self.end - 1));
        if min == max {
            min
        } else {
            rng.random_range(min..=max)
        }
    }
}

impl SizeHint for RangeInclusive<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let start = *self.start();
        let end = *self.end();
        if start > end {
            panic!(
                "size hint range {}..={} has start greater than end",
                start, end
            );
        }
        let (min, max) = clamp_bounds(start, Some(end));
        if min == max {
            min
        } else {
            rng.random_range(min..=max)
        }
    }
}

impl SizeHint for RangeFrom<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = clamp_bounds(self.start, None);
        if min == max {
            min
        } else {
            rng.random_range(min..=max)
        }
    }
}

impl SizeHint for RangeTo<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        if self.end == 0 {
            panic!("size hint range ..{} is empty", self.end);
        }
        let (min, max) = clamp_bounds(0, Some(self.end - 1));
        if min == max {
            min
        } else {
            rng.random_range(min..=max)
        }
    }
}

impl SizeHint for RangeToInclusive<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = clamp_bounds(0, Some(self.end));
        if min == max {
            min
        } else {
            rng.random_range(min..=max)
        }
    }
}

impl SizeHint for RangeFull {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = clamp_bounds(0, None);
        if min == max {
            min
        } else {
            rng.random_range(min..=max)
        }
    }
}

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

pub fn adapt_strategy<S>(strategy: S) -> IntegratedAdapter<S>
where
    S: Strategy,
    S::Value: Clone,
{
    IntegratedAdapter::new(strategy)
}

pub fn adapt<S>(strategy: S) -> IntegratedAdapter<S>
where
    S: Strategy,
    S::Value: Clone,
{
    adapt_strategy(strategy)
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
