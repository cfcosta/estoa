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

    fn min(&self) -> usize;

    fn max(&self) -> usize;

    fn to_inclusive(&self) -> RangeInclusive<usize> {
        self.min()..=self.max()
    }
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

fn validate_single(value: usize) -> usize {
    if value > COLLECTION_MAX_LEN {
        panic!(
            "size hint {} exceeds maximum supported length {}",
            value, COLLECTION_MAX_LEN
        );
    }
    value
}

fn pick_from_bounds<R: Rng + ?Sized>(
    rng: &mut R,
    min: usize,
    max: usize,
) -> usize {
    if min == max {
        min
    } else {
        rng.random_range(min..=max)
    }
}

trait Bounds {
    fn bounds(&self) -> (usize, usize);
}

impl Bounds for Range<usize> {
    fn bounds(&self) -> (usize, usize) {
        if self.start >= self.end {
            panic!("size hint range {}..{} is empty", self.start, self.end);
        }
        clamp_bounds(self.start, Some(self.end - 1))
    }
}

impl Bounds for RangeInclusive<usize> {
    fn bounds(&self) -> (usize, usize) {
        let start = *self.start();
        let end = *self.end();
        if start > end {
            panic!(
                "size hint range {}..={} has start greater than end",
                start, end
            );
        }
        clamp_bounds(start, Some(end))
    }
}

impl Bounds for RangeFrom<usize> {
    fn bounds(&self) -> (usize, usize) {
        clamp_bounds(self.start, None)
    }
}

impl Bounds for RangeTo<usize> {
    fn bounds(&self) -> (usize, usize) {
        if self.end == 0 {
            panic!("size hint range ..{} is empty", self.end);
        }
        clamp_bounds(0, Some(self.end - 1))
    }
}

impl Bounds for RangeToInclusive<usize> {
    fn bounds(&self) -> (usize, usize) {
        clamp_bounds(0, Some(self.end))
    }
}

impl SizeHint for usize {
    fn pick<R: Rng + ?Sized>(&self, _rng: &mut R) -> usize {
        validate_single(*self)
    }

    fn min(&self) -> usize {
        validate_single(*self)
    }

    fn max(&self) -> usize {
        validate_single(*self)
    }
}

impl SizeHint for Range<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = self.bounds();
        pick_from_bounds(rng, min, max)
    }

    fn min(&self) -> usize {
        self.bounds().0
    }

    fn max(&self) -> usize {
        self.bounds().1
    }
}

impl SizeHint for RangeInclusive<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = self.bounds();
        pick_from_bounds(rng, min, max)
    }

    fn min(&self) -> usize {
        self.bounds().0
    }

    fn max(&self) -> usize {
        self.bounds().1
    }
}

impl SizeHint for RangeFrom<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = self.bounds();
        pick_from_bounds(rng, min, max)
    }

    fn min(&self) -> usize {
        self.bounds().0
    }

    fn max(&self) -> usize {
        self.bounds().1
    }
}

impl SizeHint for RangeTo<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = self.bounds();
        pick_from_bounds(rng, min, max)
    }

    fn min(&self) -> usize {
        self.bounds().0
    }

    fn max(&self) -> usize {
        self.bounds().1
    }
}

impl SizeHint for RangeToInclusive<usize> {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = self.bounds();
        pick_from_bounds(rng, min, max)
    }

    fn min(&self) -> usize {
        self.bounds().0
    }

    fn max(&self) -> usize {
        self.bounds().1
    }
}

impl SizeHint for RangeFull {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let (min, max) = clamp_bounds(0, None);
        pick_from_bounds(rng, min, max)
    }

    fn min(&self) -> usize {
        clamp_bounds(0, None).0
    }

    fn max(&self) -> usize {
        clamp_bounds(0, None).1
    }
}
