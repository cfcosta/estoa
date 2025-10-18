#![allow(clippy::absurd_extreme_comparisons)]

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Mutex, OnceLock},
};

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
    if let Some(v) = val {
        assert!(v <= u8::MAX)
    }
}

#[proptest]
fn test_proptest_supports_strategy_annotations(
    #[strategy(strategies::vec(strategies::any::<u8>(), 1..=8))] items: Vec<u8>,
) {
    assert!(!items.is_empty());
}

#[proptest]
fn test_proptest_supports_mixed_arguments(
    #[strategy(strategies::vec(strategies::any::<u8>(), 1..=8))] items: Vec<u8>,
    value: u8,
) {
    assert!(!items.is_empty());
    assert!(value <= u8::MAX);
}

#[proptest]
fn test_proptest_retries_until_strategy_accepts(
    #[strategy(|generator: &mut strategies::DefaultGenerator| {
        if generator.iteration() == 0 {
            generator.reject(0u8)
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
    #[strategy(|generator: &mut strategies::DefaultGenerator| {
        generator.recurse(|nested| {
            let mut inner =
                strategies::vec(strategies::any::<u8>(), 1..=4);
            let mut outer = Vec::with_capacity(3);
            for _ in 0..3 {
                match inner(nested) {
                    strategies::Generation::Accepted { value, .. } => outer.push(value),
                    strategies::Generation::Rejected { iteration, depth, .. } => {
                        return strategies::Generation::Rejected {
                            iteration,
                            depth,
                            value: outer,
                        };
                    }
                }
            }
            nested.accept(outer)
        })
    })]
    nested: Vec<Vec<u8>>,
) {
    assert!(nested.iter().all(|inner| !inner.is_empty()));
    assert_eq!(nested.len(), 3);
}

fn case_counter() -> &'static Mutex<usize> {
    static CASE_COUNTER: OnceLock<Mutex<usize>> = OnceLock::new();
    CASE_COUNTER.get_or_init(|| Mutex::new(0))
}

#[proptest(cases = 8)]
fn test_proptest_cases_runs_body_multiple_times() {
    let mut guard = case_counter().lock().expect("case counter poisoned");
    *guard += 1;
}

#[test]
fn test_cases_configuration_runs_expected_iterations() {
    {
        let mut guard = case_counter().lock().expect("case counter poisoned");
        *guard = 0;
    }
    test_proptest_cases_runs_body_multiple_times();
    let guard = case_counter().lock().expect("case counter poisoned");
    assert_eq!(*guard, 8);
}

#[should_panic(expected = "limit 2")]
#[proptest(rejection_limit = 2)]
fn test_proptest_respects_rejection_limit_panics(
    #[strategy(|generator: &mut strategies::DefaultGenerator| {
        generator.reject(0u8)
    })]
    _value: u8,
) {
    unreachable!("strategy should always reject");
}

#[test]
fn test_rejection_limit_panics_after_expected_attempts() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        test_proptest_respects_rejection_limit_panics();
    }));
    assert!(result.is_err(), "rejection limit did not trigger panic");
}

#[should_panic(expected = "strategy recursion exceeded limit")]
#[proptest(recursion_limit = 1)]
fn test_proptest_enforces_recursion_limit(
    #[strategy(|generator: &mut strategies::DefaultGenerator| {
        generator.recurse(|outer| {
            outer.recurse(|inner| inner.accept(1usize))
        })
    })]
    _value: usize,
) {
    unreachable!("recursive strategy should exceed limit first");
}

#[test]
fn test_recursion_limit_panics_when_exceeded() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        test_proptest_enforces_recursion_limit();
    }));
    assert!(result.is_err(), "recursion limit did not trigger panic");
}
