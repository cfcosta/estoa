use estoa_proptest::strategy::{HashSetValueTree, IntValueTree, ValueTree};

fn make_tree(value: i32, shrunk: i32) -> IntValueTree<i32> {
    IntValueTree::new(value, vec![shrunk])
}

#[test]
fn hash_set_preserves_uniqueness_on_element_shrink() {
    let elements = vec![make_tree(3, 1), make_tree(1, 1)];
    let values: Vec<i32> =
        elements.iter().map(|tree| *tree.current()).collect();
    let mut tree = HashSetValueTree::from_elements(elements, values, 2);

    let _ = tree.simplify();
    let set = tree.current();
    assert_eq!(
        set.len(),
        2,
        "shrinking must not introduce duplicate values"
    );
}
