use std::ops::{
    Range,
    RangeFrom,
    RangeFull,
    RangeInclusive,
    RangeTo,
    RangeToInclusive,
};

use rand::Rng;

use crate::arbitrary::COLLECTION_MAX_LEN;

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
