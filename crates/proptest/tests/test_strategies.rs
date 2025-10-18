use std::collections::{HashMap, HashSet, VecDeque};

use estoa_proptest::{proptest, strategies::*};

#[proptest]
fn test_different(#[strategy(different::<u8>())] (a, b): (u8, u8)) {
    assert_ne!(a, b);
}

#[proptest]
fn test_vec_range(#[strategy(vec(any::<u8>(), 3..=5))] list: Vec<u8>) {
    assert!((3..=5).contains(&list.len()));
}

#[proptest]
fn test_vec_exact_size(#[strategy(vec(any::<u8>(), 4usize))] list: Vec<u8>) {
    assert_eq!(list.len(), 4);
}

#[proptest]
fn test_vec_deque(
    #[strategy(vec_deque(any::<u8>(), 4usize))] deque: VecDeque<u8>,
) {
    assert_eq!(deque.len(), 4);
}

#[proptest]
fn test_hash_set(#[strategy(hash_set(any::<u16>(), 8))] set: HashSet<u16>) {
    assert_eq!(set.len(), 8);
}

#[proptest]
fn test_hash_map(
    #[strategy(hash_map(
        any::<u8>(),
        any::<u8>(),
        6
    ))]
    map: HashMap<u8, u8>,
) {
    assert_eq!(map.len(), 6);
    assert_eq!(map.keys().len(), 6);
}
