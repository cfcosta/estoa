use std::collections::BinaryHeap;

use estoa_proptest::Arbitrary;
use rand::{CryptoRng, RngCore};

#[derive(Default)]
struct MaxRng;

impl RngCore for MaxRng {
    fn next_u32(&mut self) -> u32 {
        u32::MAX
    }

    fn next_u64(&mut self) -> u64 {
        u64::MAX
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(u8::MAX);
    }
}

impl CryptoRng for MaxRng {}

#[test]
fn binary_heap_arbitrary_can_be_non_empty_with_deterministic_rng() {
    let heap = BinaryHeap::<u8>::arbitrary(&mut MaxRng);

    assert!(
        !heap.is_empty(),
        "deterministic RNG should yield a non-empty heap"
    );
}
