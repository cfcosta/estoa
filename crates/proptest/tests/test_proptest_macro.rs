use estoa_proptest::{Arbitrary, proptest};
use rand::Rng;

struct Bounded {
    upper: u16,
    lower: u16,
}

impl Arbitrary for Bounded {
    fn arbitrary<R: rand::RngCore + rand::CryptoRng + ?Sized>(
        rng: &mut R,
    ) -> Self {
        let upper = u16::arbitrary(rng).max(1);
        let lower = rng.random_range(0..=upper);
        Self { upper, lower }
    }
}

#[proptest]
fn test_bounded_value_is_within_range(bounded: Bounded) {
    assert!(bounded.lower <= bounded.upper);
}

#[proptest]
fn test_proptest_supports_multiple_arguments(value: u8, text: String) {
    assert!(value <= u8::MAX);
    assert!(text.capacity() >= text.len());
}

#[proptest]
fn proptest_supports_generic_arguments(val: Option<u8>) {
    match val {
        Some(v) => assert!(v <= u8::MAX),
        None => {}
    }
}
