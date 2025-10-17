use rand::{CryptoRng, RngCore};

use crate::{Arbitrary, arbitrary};

mod generator;

pub use generator::*;

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
