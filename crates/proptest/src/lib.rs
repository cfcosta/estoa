use rand::{CryptoRng, RngCore, rngs::ThreadRng};

mod arbitrary;
pub mod strategies;
pub mod strategy;

pub use arbitrary::Arbitrary;
pub use estoa_proptest_macros::proptest;

pub fn random<T: Arbitrary>() -> strategies::Generation<T> {
    T::random()
}

pub fn arbitrary<T: Arbitrary, R: RngCore + CryptoRng>(
    generator: &mut strategies::Generator<R>,
) -> strategies::Generation<T> {
    T::generate(generator)
}

pub fn rng() -> ThreadRng {
    rand::rng()
}
