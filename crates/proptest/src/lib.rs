use rand::{CryptoRng, RngCore, rngs::ThreadRng};

mod arbitrary;
pub mod strategy;

pub use arbitrary::Arbitrary;
pub use estoa_proptest_macros::proptest;
pub use strategy::runtime::{
    ConstantValueTree,
    DefaultGenerator,
    Generation,
    Generator,
    IntegratedAdapter,
    MAX_STRATEGY_ATTEMPTS,
    SizeHint,
    adapt,
    adapt_strategy,
    build_default_generator,
    execute,
    from_arbitrary,
};

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
