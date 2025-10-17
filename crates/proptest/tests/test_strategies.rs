use std::collections::{HashMap, HashSet};

use estoa_proptest::{proptest, strategies};

#[proptest]
fn test_not_equal(#[strategy(strategies::different)] (a, b): (u8, u8)) {
    assert_ne!(a, b);
}

#[proptest]
fn test_vec_not_empty(#[strategy(strategies::vec::not_empty)] list: Vec<u8>) {
    assert_ne!(list.len(), 0);
}

#[proptest]
fn test_hash_set_not_empty(
    #[strategy(strategies::hash_set::not_empty)] set: HashSet<u16>,
) {
    assert!(!set.is_empty());
    let unique = set.iter().copied().collect::<HashSet<_>>();
    assert_eq!(set.len(), unique.len());
}

#[proptest]
fn test_hash_map_not_empty(
    #[strategy(strategies::hash_map::not_empty)] map: HashMap<u8, u8>,
) {
    assert!(!map.is_empty());
    let unique_keys = map.keys().copied().collect::<HashSet<_>>();
    assert_eq!(map.len(), unique_keys.len());
}
