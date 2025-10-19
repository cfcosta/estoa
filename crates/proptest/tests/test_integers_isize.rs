use estoa_proptest::strategy::{
    AnyIsize,
    Strategy,
    ValueTree,
    runtime::{Generation, Generator},
};

fn final_value_after_all_simplifies<T>(mut tree: T) -> T::Value
where
    T: ValueTree,
    T::Value: Copy,
{
    while tree.simplify() {}
    *tree.current()
}

#[test]
fn any_isize_shrinks_to_zero_when_range_spans_zero() {
    let mut strategy = AnyIsize::new(-10..=10);
    let mut generator = Generator::build_with_limit(rand::rng(), usize::MAX);

    let tree = match strategy.new_tree(&mut generator) {
        Generation::Accepted { value, .. } => value,
        Generation::Rejected { .. } => panic!("strategy rejected"),
    };

    assert_eq!(final_value_after_all_simplifies(tree), 0);
}

#[test]
fn any_isize_shrinks_to_lower_bound_when_positive_only() {
    let mut strategy = AnyIsize::new(5..=12);
    let mut generator = Generator::build_with_limit(rand::rng(), usize::MAX);

    let tree = match strategy.new_tree(&mut generator) {
        Generation::Accepted { value, .. } => value,
        Generation::Rejected { .. } => panic!("strategy rejected"),
    };

    let final_value = final_value_after_all_simplifies(tree);
    assert!((5..=12).contains(&final_value));
    assert_eq!(final_value, 5);
}

#[test]
fn any_isize_shrinks_to_upper_bound_when_negative_only() {
    let mut strategy = AnyIsize::new(-12..=-5);
    let mut generator = Generator::build_with_limit(rand::rng(), usize::MAX);

    let tree = match strategy.new_tree(&mut generator) {
        Generation::Accepted { value, .. } => value,
        Generation::Rejected { .. } => panic!("strategy rejected"),
    };

    let final_value = final_value_after_all_simplifies(tree);
    assert!((-12..=-5).contains(&final_value));
    assert_eq!(final_value, -5);
}
