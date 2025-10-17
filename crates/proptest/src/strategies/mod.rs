use rand::{CryptoRng, RngCore};

use crate::{Arbitrary, arbitrary};

pub enum Generation<T> {
    Accepted(T),
    Rejected(T),
}

impl<T> Generation<T> {
    pub fn take(self) -> T {
        match self {
            Generation::Accepted(v) => v,
            Generation::Rejected(v) => v,
        }
    }
}

pub fn different<T: Arbitrary + PartialEq, R: RngCore + CryptoRng + ?Sized>(
    rng: &mut R,
) -> Generation<(T, T)> {
    let (a, b) = (arbitrary(rng), arbitrary(rng));

    if a != b {
        Generation::Accepted((a, b))
    } else {
        Generation::Rejected((a, b))
    }
}

pub mod vec {
    use rand::{CryptoRng, Rng, RngCore};

    use super::Generation;
    use crate::{Arbitrary, arbitrary, arbitrary::COLLECTION_MAX_LEN};

    pub fn not_empty<
        T: Arbitrary + PartialEq,
        R: RngCore + CryptoRng + ?Sized,
    >(
        rng: &mut R,
    ) -> Generation<Vec<T>> {
        let len = rng.random_range(1..=COLLECTION_MAX_LEN);

        Generation::Accepted(
            (0..len).into_iter().map(|_| arbitrary(rng)).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::ThreadRng;

    use super::*;

    #[test]
    fn test_not_equal() {
        let mut rng = ThreadRng::default();

        match different::<u8, _>(&mut rng) {
            Generation::Accepted((a, b)) => assert_ne!(a, b),
            Generation::Rejected((a, b)) => assert_eq!(a, b),
        }
    }

    #[test]
    fn test_vec_not_empty() {
        let mut rng = ThreadRng::default();
        let items: Vec<u8> = vec::not_empty(&mut rng).take();

        assert_ne!(items.len(), 0);
    }
}
