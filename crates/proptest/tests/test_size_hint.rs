use estoa_proptest::strategy::{
    AnyU8,
    Strategy,
    ValueTree,
    VecStrategy,
    runtime::{Generation, Generator},
};

#[test]
fn inclusive_range_length_is_respected() {
    let mut strategy = VecStrategy::new(AnyU8::default(), 3usize..=5usize);
    let mut generator = Generator::build(rand::rng());

    let value_tree = match strategy.new_tree(&mut generator) {
        Generation::Accepted { value, .. } => value,
        Generation::Rejected { .. } => panic!("strategy rejected"),
    };

    let len = value_tree.current().len();
    assert!((3..=5).contains(&len));
}

#[test]
#[should_panic]
fn range_to_zero_panics_via_vec_strategy() {
    let _ = VecStrategy::new(AnyU8::default(), ..0usize);
}

#[test]
fn fixed_exact_size_respected() {
    let mut strategy = VecStrategy::new(AnyU8::default(), 4usize..=4usize);
    let mut generator = Generator::build(rand::rng());
    let value_tree = match strategy.new_tree(&mut generator) {
        Generation::Accepted { value, .. } => value,
        Generation::Rejected { .. } => panic!("strategy rejected"),
    };

    assert_eq!(value_tree.current().len(), 4);
}
