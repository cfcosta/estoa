use estoa_proptest::strategy::{
    AnyChar,
    Strategy,
    ValueTree,
    runtime::{Generation, Generator},
};

fn collect_candidates(mut tree: impl ValueTree<Value = char>) -> Vec<char> {
    let mut out = Vec::new();
    while tree.simplify() {
        out.push(*tree.current());
    }
    out
}

#[test]
fn char_candidates_do_not_include_original() {
    let mut strategy = AnyChar::new('0'..='9');
    let mut generator = Generator::build(rand::rng());
    let tree = match strategy.new_tree(&mut generator) {
        Generation::Accepted { value, .. } => value,
        Generation::Rejected { .. } => panic!("strategy rejected"),
    };

    let original = *tree.current();
    let candidates = collect_candidates(tree);
    assert!(
        !candidates.contains(&original),
        "original value must not be among candidates"
    );
}
