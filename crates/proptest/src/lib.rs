use rand::{CryptoRng, RngCore, rngs::ThreadRng};

mod arbitrary;
pub mod strategy;

pub use arbitrary::Arbitrary;
pub use estoa_proptest_macros::proptest;
pub use strategy::{SizeHint, runtime::*};

pub fn random<T: Arbitrary>() -> strategy::runtime::Generation<T> {
    T::random()
}

pub fn arbitrary<T: Arbitrary, R: RngCore + CryptoRng>(
    generator: &mut strategy::runtime::Generator<R>,
) -> strategy::runtime::Generation<T> {
    T::generate(generator)
}

pub fn rng() -> ThreadRng {
    rand::rng()
}
