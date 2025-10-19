use std::collections::HashSet;

use estoa_proptest::strategy::{HashMapValueTree, IntValueTree, ValueTree};

fn key_tree(value: i32, shrunk: i32) -> IntValueTree<i32> {
    IntValueTree::new(value, vec![shrunk])
}

fn value_tree(value: i32) -> IntValueTree<i32> {
    IntValueTree::new(value, Vec::new())
}

#[test]
fn hash_map_rejects_duplicate_key_after_shrink() {
    let entries = vec![
        (key_tree(5, 3), value_tree(10)),
        (key_tree(3, 3), value_tree(20)),
    ];
    let keys: Vec<i32> = entries.iter().map(|(k, _)| *k.current()).collect();
    let values: Vec<i32> = entries.iter().map(|(_, v)| *v.current()).collect();
    let mut tree = HashMapValueTree::from_entries(
        entries, keys, values, /*min_len*/ 2,
    );

    let _ = tree.simplify();
    let map = tree.current();
    let unique: HashSet<_> = map.keys().copied().collect();
    assert_eq!(
        unique.len(),
        map.len(),
        "shrinking must not introduce duplicate keys"
    );
}

#[test]
fn hash_map_complicate_after_length_removal_restores_length() {
    let entries = vec![
        (key_tree(9, 8), value_tree(1)),
        (key_tree(7, 6), value_tree(2)),
        (key_tree(5, 4), value_tree(3)),
    ];
    let keys: Vec<i32> = entries.iter().map(|(k, _)| *k.current()).collect();
    let values: Vec<i32> = entries.iter().map(|(_, v)| *v.current()).collect();
    let mut tree = HashMapValueTree::from_entries(entries, keys, values, 0);

    assert!(tree.simplify(), "expected a length-removal step");
    let reduced_len = tree.current().len();
    assert!(reduced_len < 3);
    let _ = tree.complicate();
    assert!(
        tree.current().len() >= reduced_len,
        "complicate should not decrease length further"
    );
}
