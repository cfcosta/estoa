use rand::{CryptoRng, RngCore, rngs::ThreadRng};

mod arbitrary;
pub mod strategies;

pub use arbitrary::Arbitrary;
pub use estoa_proptest_macros::proptest;

pub fn random<T: Arbitrary>() -> T {
    T::random()
}

pub fn arbitrary<T: Arbitrary, R: RngCore + CryptoRng + ?Sized>(
    rng: &mut R,
) -> T {
    T::arbitrary(rng)
}

pub fn rng() -> ThreadRng {
    rand::rng()
}
