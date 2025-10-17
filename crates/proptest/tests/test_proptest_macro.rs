use estoa_proptest::{Arbitrary, proptest, strategies};
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
fn test_proptest_supports_generic_arguments(val: Option<u8>) {
    match val {
        Some(v) => assert!(v <= u8::MAX),
        None => {}
    }
}

#[proptest]
fn test_proptest_supports_strategy_annotations(
    #[strategy(strategies::vec::not_empty)] items: Vec<u8>,
) {
    assert!(!items.is_empty());
}

#[proptest]
fn test_proptest_supports_mixed_arguments(
    #[strategy(strategies::vec::not_empty)] items: Vec<u8>,
    value: u8,
) {
    assert!(!items.is_empty());
    assert!(value <= u8::MAX);
}

#[proptest]
fn test_proptest_retries_until_strategy_accepts(
    #[strategy(|generator: &mut strategies::Generator<rand::rngs::ThreadRng>| {
        if generator.iteration() == 0 {
            let discarded = estoa_proptest::arbitrary(&mut generator.rng);
            generator.reject(discarded)
        } else {
            generator.accept(42u8)
        }
    })]
    value: u8,
) {
    assert_eq!(value, 42);
}

#[proptest]
fn test_proptest_handles_recursive_generators(
    #[strategy(|generator: &mut strategies::Generator<rand::rngs::ThreadRng>| {
        generator.recurse(|nested| {
            let mut outer = Vec::new();
            for _ in 0..3 {
                let inner = strategies::vec::not_empty(nested).take();
                outer.push(inner);
            }
            nested.accept(outer)
        })
    })]
    nested: Vec<Vec<u8>>,
) {
    assert_eq!(nested.len(), 3);
    assert!(nested.iter().all(|inner| !inner.is_empty()));
}
