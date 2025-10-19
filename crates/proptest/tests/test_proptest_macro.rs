#![allow(clippy::absurd_extreme_comparisons)]

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Mutex, OnceLock},
};

use estoa_proptest::{
    Arbitrary,
    proptest,
    strategy::{
        runtime::{Generation, Generator},
        *,
    },
};
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
    #[strategy(VecStrategy::new(AnyU8::default(), 1usize..=8usize))] items: Vec<
        u8,
    >,
) {
    assert!(!items.is_empty());
}

#[proptest]
fn test_proptest_supports_mixed_arguments(
    #[strategy(VecStrategy::new(AnyU8::default(), 1usize..=8usize))] items: Vec<
        u8,
    >,
    value: u8,
) {
    assert!(!items.is_empty());
    assert!(value <= u8::MAX);
}

#[derive(Default)]
struct RetryStrategy {
    rejected: bool,
}

impl Strategy for RetryStrategy {
    type Value = u8;
    type Tree = StaticTree<u8>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        if !self.rejected {
            self.rejected = true;
            generator.reject(StaticTree::new(0))
        } else {
            generator.accept(StaticTree::new(42))
        }
    }
}

#[proptest]
fn test_proptest_retries_until_strategy_accepts(
    #[strategy(RetryStrategy::default())] value: u8,
) {
    assert_eq!(value, 42);
}

#[derive(Default)]
struct NestedVecStrategy;

impl Strategy for NestedVecStrategy {
    type Value = Vec<Vec<u8>>;
    type Tree = StaticTree<Vec<Vec<u8>>>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        generator.recurse(|outer| {
            let mut values = Vec::with_capacity(3);
            for _ in 0..3 {
                let mut inner = Vec::with_capacity(4);
                for _ in 0..4 {
                    inner.push(outer.rng.random::<u8>());
                }
                values.push(inner);
            }
            outer.accept(StaticTree::new(values))
        })
    }
}

#[proptest]
fn test_proptest_handles_recursive_generators(
    #[strategy(NestedVecStrategy)] nested: Vec<Vec<u8>>,
) {
    assert!(nested.iter().all(|inner| inner.len() == 4));
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

#[derive(Default)]
struct AlwaysReject;

impl Strategy for AlwaysReject {
    type Value = u8;
    type Tree = StaticTree<u8>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        generator.reject(StaticTree::new(0))
    }
}

#[should_panic(expected = "limit 2")]
#[proptest(rejection_limit = 2)]
fn test_proptest_respects_rejection_limit_panics(
    #[strategy(AlwaysReject)] _value: u8,
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

#[derive(Default)]
struct RecursiveOverflow;

impl Strategy for RecursiveOverflow {
    type Value = usize;
    type Tree = StaticTree<usize>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        generator.recurse(|outer| {
            outer.recurse(|inner| inner.accept(StaticTree::new(1usize)))
        })
    }
}

#[should_panic(expected = "strategy recursion exceeded limit")]
#[proptest(recursion_limit = 1)]
fn test_proptest_enforces_recursion_limit(
    #[strategy(RecursiveOverflow)] _value: usize,
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
