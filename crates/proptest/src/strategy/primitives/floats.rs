use core::cmp::Ordering;

use rand::Rng;

use crate::{
    strategies::{Generation, Generator},
    strategy::{Strategy, ValueTree},
};

const MAX_FLOAT_SIMPLIFY_STEPS: usize = 64;

fn canonical_zero<T: PartialEq + Copy>(value: T, zero: T) -> T {
    if value == zero { zero } else { value }
}

fn approx_eq(value: f64, other: f64) -> bool {
    (value - other).abs()
        <= f64::EPSILON * (value.abs().max(other.abs()).max(1.0))
}

fn push_candidate<T: PartialEq + Copy>(candidates: &mut Vec<T>, candidate: T) {
    if candidates.last().copied() != Some(candidate) {
        candidates.push(candidate);
    }
}

fn float_anchor(lo: f64, hi: f64) -> f64 {
    match (lo.partial_cmp(&0.0), hi.partial_cmp(&0.0)) {
        (Some(Ordering::Greater), _) => lo,
        (_, Some(Ordering::Less)) => hi,
        _ => 0.0,
    }
}

fn build_float_candidates(value: f64, target: f64) -> Vec<f64> {
    let mut candidates = Vec::new();
    if value.is_nan() {
        if target == 0.0 {
            candidates.push(0.0);
        } else {
            candidates.push(target);
        }
        return candidates;
    }

    let mut current = canonical_zero(value, 0.0);
    let target = canonical_zero(target, 0.0);

    if approx_eq(current, target) {
        return candidates;
    }

    for _ in 0..MAX_FLOAT_SIMPLIFY_STEPS {
        let delta = current - target;
        let next = canonical_zero(current - delta / 2.0, 0.0);
        if approx_eq(next, current) {
            break;
        }

        push_candidate(&mut candidates, next);
        current = next;
        if approx_eq(current, target) {
            break;
        }
    }

    if !candidates.is_empty() {
        if !approx_eq(*candidates.last().unwrap(), target) {
            push_candidate(&mut candidates, target);
        }
    } else {
        push_candidate(&mut candidates, target);
    }

    candidates
}

pub struct FloatValueTree<T>
where
    T: Copy + PartialEq,
{
    current: T,
    history: Vec<T>,
    candidates: Vec<T>,
    index: usize,
}

impl<T> FloatValueTree<T>
where
    T: Copy + PartialEq,
{
    pub fn new(current: T, candidates: Vec<T>) -> Self {
        Self {
            current,
            history: Vec::new(),
            candidates,
            index: 0,
        }
    }
}

impl<T> ValueTree for FloatValueTree<T>
where
    T: Copy + PartialEq,
{
    type Value = T;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        let candidate = match self.candidates.get(self.index) {
            Some(candidate) => *candidate,
            None => return false,
        };

        self.history.push(self.current);
        self.current = candidate;
        self.index += 1;
        true
    }

    fn complicate(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };

        self.current = previous;
        self.index < self.candidates.len()
    }
}

macro_rules! impl_float_strategy {
    ($name:ident, $ty:ty, $zero:expr) => {
        #[derive(Clone)]
        pub struct $name {
            range: core::ops::RangeInclusive<$ty>,
        }

        impl $name {
            pub fn new(range: core::ops::RangeInclusive<$ty>) -> Self {
                Self { range }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(<$ty>::MIN..=<$ty>::MAX)
            }
        }

        impl Strategy for $name {
            type Value = $ty;
            type Tree = FloatValueTree<$ty>;

            fn new_tree<R: rand::RngCore + rand::CryptoRng>(
                &mut self,
                generator: &mut Generator<R>,
            ) -> Generation<Self::Tree> {
                let value = canonical_zero(
                    generator.rng.random_range(self.range.clone()),
                    $zero,
                );
                let lo = *self.range.start() as f64;
                let hi = *self.range.end() as f64;
                let target = float_anchor(lo, hi);
                let candidates = build_float_candidates(value as f64, target);
                let candidates = candidates
                    .into_iter()
                    .filter_map(|candidate| {
                        let candidate = canonical_zero(candidate as $ty, $zero);
                        if self.range.contains(&candidate) {
                            Some(candidate)
                        } else {
                            None
                        }
                    })
                    .collect();

                generator.accept(FloatValueTree::new(value, candidates))
            }
        }
    };
}

impl_float_strategy!(AnyF32, f32, 0.0f32);
impl_float_strategy!(AnyF64, f64, 0.0f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_shrink_toward_zero() {
        let candidates = build_float_candidates(32.0, 0.0);
        assert!(candidates.windows(2).all(|w| w[0].abs() >= w[1].abs()));
        assert_eq!(candidates.last().copied(), Some(0.0));
    }

    #[test]
    fn floats_respect_bounds() {
        let range = 5.0f64..=10.0f64;
        let target = float_anchor(*range.start(), *range.end());
        let candidates = build_float_candidates(9.0, target);
        assert!(candidates.iter().all(|candidate| range.contains(candidate)));
        assert_eq!(candidates.last().copied(), Some(5.0));
    }

    #[test]
    fn float_value_tree_complicates() {
        let mut tree = FloatValueTree::new(8.0f32, vec![4.0, 2.0, 0.0]);
        assert!(tree.simplify());
        assert_eq!(*tree.current(), 4.0);
        assert!(tree.complicate());
        assert_eq!(*tree.current(), 8.0);
        assert!(tree.simplify());
        assert_eq!(*tree.current(), 2.0);
    }
}
