pub use rand::{CryptoRng, RngCore};
use rand::{
    Rng,
    distr::{SampleString, StandardUniform},
};

const STRING_MAX_LEN: usize = 1024;

pub trait Arbitrary
where
    Self: Sized,
{
    fn arbitrary<R: RngCore + CryptoRng>(rng: &mut R) -> Self;

    fn random() -> Self {
        Self::arbitrary(&mut rand::rng())
    }
}

impl Arbitrary for String {
    fn arbitrary<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=STRING_MAX_LEN);
        StandardUniform.sample_string(rng, len)
    }
}

impl Arbitrary for u32 {
    fn arbitrary<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        rng.next_u32()
    }
}

impl Arbitrary for u64 {
    fn arbitrary<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        rng.next_u64()
    }
}

impl Arbitrary for usize {
    fn arbitrary<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        rng.random_range(0..usize::MAX)
    }
}
