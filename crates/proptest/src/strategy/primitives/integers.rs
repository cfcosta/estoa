use std::ops::RangeInclusive;

use rand::Rng;

use crate::strategy::{
    Strategy,
    ValueTree,
    runtime::{Generation, Generator},
};

pub struct IntValueTree<T>
where
    T: Copy,
{
    current: T,
    history: Vec<T>,
    candidates: Vec<T>,
    next_index: usize,
}

impl<T> IntValueTree<T>
where
    T: Copy,
{
    pub fn new(current: T, candidates: Vec<T>) -> Self {
        Self {
            current,
            history: Vec::new(),
            candidates,
            next_index: 0,
        }
    }
}

impl<T> ValueTree for IntValueTree<T>
where
    T: Copy,
{
    type Value = T;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        let candidate = match self.candidates.get(self.next_index) {
            Some(candidate) => *candidate,
            None => return false,
        };

        self.history.push(self.current);
        self.current = candidate;
        self.next_index += 1;
        true
    }

    fn complicate(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };

        self.current = previous;
        self.next_index < self.candidates.len()
    }
}

macro_rules! impl_signed_int_strategy {
    ($name:ident, $ty:ty, $zero:expr) => {
        #[derive(Clone)]
        pub struct $name {
            range: RangeInclusive<$ty>,
        }

        impl $name {
            pub fn new(range: RangeInclusive<$ty>) -> Self {
                Self { range }
            }

            #[inline]
            fn anchor(lo: $ty, hi: $ty) -> $ty {
                if lo <= $zero && hi >= $zero {
                    $zero
                } else if lo > $zero {
                    lo
                } else {
                    hi
                }
            }

            fn build_candidates(value: $ty, target: $ty) -> Vec<$ty> {
                let mut current = value as i128;
                let target = target as i128;
                let mut candidates = Vec::new();

                while current != target {
                    let delta = current - target;
                    let magnitude = if delta >= 0 {
                        delta as u128
                    } else if delta == i128::MIN {
                        (i128::MAX as u128) + 1
                    } else {
                        (-delta) as u128
                    };

                    let mut step = magnitude / 2;
                    if step == 0 {
                        step = 1;
                    }

                    let direction = if delta >= 0 { 1 } else { -1 };
                    let step = step as i128 * direction;
                    let next = current - step;

                    if next == current {
                        break;
                    }

                    candidates.push(next as $ty);
                    current = next;
                }

                candidates
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(<$ty>::MIN..=<$ty>::MAX)
            }
        }

        impl Strategy for $name {
            type Value = $ty;
            type Tree = IntValueTree<$ty>;

            fn new_tree<R: rand::RngCore + rand::CryptoRng>(
                &mut self,
                generator: &mut Generator<R>,
            ) -> Generation<Self::Tree> {
                let value = generator.rng.random_range(self.range.clone());
                let lo = *self.range.start();
                let hi = *self.range.end();
                let target = Self::anchor(lo, hi);
                let candidates = Self::build_candidates(value, target);
                generator.accept(IntValueTree::new(value, candidates))
            }
        }
    };
}

macro_rules! impl_unsigned_int_strategy {
    ($name:ident, $ty:ty) => {
        #[derive(Clone)]
        pub struct $name {
            range: RangeInclusive<$ty>,
        }

        impl $name {
            pub fn new(range: RangeInclusive<$ty>) -> Self {
                Self { range }
            }

            #[inline]
            fn anchor(lo: $ty) -> $ty {
                if lo == 0 { 0 } else { lo }
            }

            fn build_candidates(value: $ty, target: $ty) -> Vec<$ty> {
                let mut current = value as u128;
                let target = target as u128;
                let mut candidates = Vec::new();

                while current != target {
                    let diff = if current >= target {
                        current - target
                    } else {
                        target - current
                    };

                    let mut step = diff / 2;
                    if step == 0 {
                        step = 1;
                    }

                    let next = if current >= target {
                        current - step
                    } else {
                        current + step
                    };

                    if next == current {
                        break;
                    }

                    candidates.push(next as $ty);
                    current = next;
                }

                candidates
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(<$ty>::MIN..=<$ty>::MAX)
            }
        }

        impl Strategy for $name {
            type Value = $ty;
            type Tree = IntValueTree<$ty>;

            fn new_tree<R: rand::RngCore + rand::CryptoRng>(
                &mut self,
                generator: &mut Generator<R>,
            ) -> Generation<Self::Tree> {
                let value = generator.rng.random_range(self.range.clone());
                let lo = *self.range.start();
                let target = Self::anchor(lo);
                let candidates = Self::build_candidates(value, target);
                generator.accept(IntValueTree::new(value, candidates))
            }
        }
    };
}

impl_signed_int_strategy!(AnyI8, i8, 0i8);
impl_signed_int_strategy!(AnyI16, i16, 0i16);
impl_signed_int_strategy!(AnyI32, i32, 0i32);
impl_signed_int_strategy!(AnyI64, i64, 0i64);
impl_signed_int_strategy!(AnyI128, i128, 0i128);

impl_unsigned_int_strategy!(AnyU8, u8);
impl_unsigned_int_strategy!(AnyU16, u16);
impl_unsigned_int_strategy!(AnyU32, u32);
impl_unsigned_int_strategy!(AnyU64, u64);
impl_unsigned_int_strategy!(AnyU128, u128);

#[derive(Clone)]
pub struct AnyIsize {
    range: RangeInclusive<isize>,
}

impl AnyIsize {
    pub fn new(range: RangeInclusive<isize>) -> Self {
        Self { range }
    }

    #[inline]
    fn anchor(lo: isize, hi: isize) -> isize {
        if lo <= 0 && hi >= 0 {
            0
        } else if lo > 0 {
            lo
        } else {
            hi
        }
    }

    fn build_candidates(value: isize, target: isize) -> Vec<isize> {
        let mut current = value as i128;
        let target = target as i128;
        let mut candidates = Vec::new();

        while current != target {
            let delta = current - target;
            let magnitude = if delta >= 0 {
                delta as u128
            } else if delta == i128::MIN {
                (i128::MAX as u128) + 1
            } else {
                (-delta) as u128
            };

            let mut step = magnitude / 2;
            if step == 0 {
                step = 1;
            }

            let direction = if delta >= 0 { 1 } else { -1 };
            let step = step as i128 * direction;
            let next = current - step;

            if next == current {
                break;
            }

            candidates.push(next as isize);
            current = next;
        }

        candidates
    }

    pub(crate) fn sample<R: rand::RngCore + rand::CryptoRng>(
        rng: &mut R,
        range: RangeInclusive<isize>,
    ) -> isize {
        let lo = *range.start();
        let hi = *range.end();

        if lo == hi {
            return lo;
        }

        let lo_i = lo as i128;
        let hi_i = hi as i128;
        let span = (hi_i - lo_i) as u128 + 1;
        let offset = rng.random_range(0..span) as i128;
        (lo_i + offset) as isize
    }
}

impl Default for AnyIsize {
    fn default() -> Self {
        Self::new(isize::MIN..=isize::MAX)
    }
}

impl Strategy for AnyIsize {
    type Value = isize;
    type Tree = IntValueTree<isize>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let value = Self::sample(&mut generator.rng, self.range.clone());
        let lo = *self.range.start();
        let hi = *self.range.end();
        let target = Self::anchor(lo, hi);
        let candidates = Self::build_candidates(value, target);
        generator.accept(IntValueTree::new(value, candidates))
    }
}

#[derive(Clone)]
pub struct AnyUsize {
    range: RangeInclusive<usize>,
}

impl AnyUsize {
    pub fn new(range: RangeInclusive<usize>) -> Self {
        Self { range }
    }

    #[inline]
    fn anchor(lo: usize) -> usize {
        if lo == 0 { 0 } else { lo }
    }

    fn build_candidates(value: usize, target: usize) -> Vec<usize> {
        let mut current = value as u128;
        let target = target as u128;
        let mut candidates = Vec::new();

        while current != target {
            let diff = current.abs_diff(target);

            let mut step = diff / 2;
            if step == 0 {
                step = 1;
            }

            let next = if current >= target {
                current - step
            } else {
                current + step
            };

            if next == current {
                break;
            }

            candidates.push(next as usize);
            current = next;
        }

        candidates
    }

    pub(crate) fn sample<R: rand::RngCore + rand::CryptoRng>(
        rng: &mut R,
        range: RangeInclusive<usize>,
    ) -> usize {
        let lo = *range.start();
        let hi = *range.end();

        if lo == hi {
            return lo;
        }

        let lo_u = lo as u128;
        let hi_u = hi as u128;
        let span = (hi_u - lo_u) + 1;
        let offset = rng.random_range(0..span);
        (lo_u + offset) as usize
    }
}

impl Default for AnyUsize {
    fn default() -> Self {
        Self::new(usize::MIN..=usize::MAX)
    }
}

impl Strategy for AnyUsize {
    type Value = usize;
    type Tree = IntValueTree<usize>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let value = Self::sample(&mut generator.rng, self.range.clone());
        let lo = *self.range.start();
        let target = Self::anchor(lo);
        let candidates = Self::build_candidates(value, target);
        generator.accept(IntValueTree::new(value, candidates))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_sequence_shrinks_toward_zero() {
        let candidates = AnyI32::build_candidates(23, 0);
        assert_eq!(candidates, vec![12, 6, 3, 2, 1, 0]);
    }

    #[test]
    fn signed_sequence_respects_positive_bounds() {
        let target = AnyI32::anchor(5, 10);
        let candidates = AnyI32::build_candidates(9, target);
        assert_eq!(target, 5);
        assert_eq!(candidates, vec![7, 6, 5]);
    }

    #[test]
    fn signed_sequence_respects_negative_bounds() {
        let target = AnyI32::anchor(-10, -5);
        let candidates = AnyI32::build_candidates(-9, target);
        assert_eq!(target, -5);
        assert_eq!(candidates, vec![-7, -6, -5]);
    }

    #[test]
    fn unsigned_sequence_shrinks_to_zero_when_available() {
        let target = AnyU32::anchor(0);
        let candidates = AnyU32::build_candidates(9, target);
        assert_eq!(target, 0);
        assert_eq!(candidates, vec![5, 3, 2, 1, 0]);
    }

    #[test]
    fn unsigned_sequence_respects_lower_bound() {
        let target = AnyU32::anchor(5);
        let candidates = AnyU32::build_candidates(9, target);
        assert_eq!(target, 5);
        assert_eq!(candidates, vec![7, 6, 5]);
    }

    #[test]
    fn complicate_restarts_from_previous_candidate() {
        let mut tree = IntValueTree::new(8u32, vec![4, 2, 1]);

        assert!(tree.simplify());
        assert_eq!(*tree.current(), 4);

        assert!(tree.simplify());
        assert_eq!(*tree.current(), 2);

        assert!(tree.complicate());
        assert_eq!(*tree.current(), 4);

        assert!(tree.simplify());
        assert_eq!(*tree.current(), 1);

        assert!(!tree.complicate());
        assert_eq!(*tree.current(), 4);
    }
}
