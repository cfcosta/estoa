use estoa_proptest::strategy::{
    AnyF64,
    Strategy,
    ValueTree,
    runtime::{Generation, Generator},
};

#[test]
fn floats_shrink_monotonically_and_end_at_anchor() {
    let mut strategy = AnyF64::new(5.0..=10.0);
    let mut generator = Generator::build_with_limit(rand::rng(), usize::MAX);

    let mut tree = match strategy.new_tree(&mut generator) {
        Generation::Accepted { value, .. } => value,
        Generation::Rejected { .. } => panic!("strategy rejected"),
    };

    let mut prev_abs = f64::INFINITY;
    let mut final_value = *tree.current();

    while tree.simplify() {
        let current = *tree.current();
        assert!(
            current.abs() <= prev_abs + f64::EPSILON,
            "candidate should move monotonically toward anchor"
        );
        prev_abs = current.abs();
        final_value = current;
    }

    assert_eq!(final_value, 5.0);
}
