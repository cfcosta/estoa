use estoa_proptest::strategy::{
    AnyString,
    IntValueTree,
    Strategy,
    StringValueTree,
    ValueTree,
    runtime::{Generation, Generator},
};

fn char_tree(c: char) -> IntValueTree<char> {
    IntValueTree::new(c, Vec::new())
}

#[test]
fn string_never_shrinks_below_min_len() {
    let mut tree = StringValueTree::from_trees(vec![char_tree('x')], 1);
    let _ = tree.simplify();
    assert_eq!(tree.current(), "x");
}

#[test]
fn any_string_respects_fixed_len_hint() {
    let mut strategy = AnyString::new(4usize..=4usize);
    let mut generator = Generator::build_with_limit(rand::rng(), usize::MAX);
    let value_tree: StringValueTree = match strategy.new_tree(&mut generator) {
        Generation::Accepted { value, .. } => value,
        Generation::Rejected { .. } => {
            panic!("strategy rejected during generation")
        }
    };
    assert_eq!(value_tree.current().chars().count(), 4);
}
