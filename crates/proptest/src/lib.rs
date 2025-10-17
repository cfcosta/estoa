use rand::{CryptoRng, RngCore};

mod arbitrary;

pub use arbitrary::Arbitrary;

pub fn random<T: Arbitrary>() -> T {
    T::random()
}

pub fn arbitrary<T: Arbitrary, R: RngCore + CryptoRng + ?Sized>(
    rng: &mut R,
) -> T {
    T::arbitrary(rng)
}
