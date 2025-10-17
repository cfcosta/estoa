use estoa_proptest::{proptest, strategies};

#[proptest]
fn test_not_equal(#[strategy(strategies::different)] (a, b): (u8, u8)) {
    assert_ne!(a, b);
}

#[proptest]
fn test_vec_not_empty(#[strategy(strategies::vec::not_empty)] list: Vec<u8>) {
    assert_ne!(list.len(), 0);
}
