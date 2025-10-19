use estoa_proptest::strategy::{ValueTree, VecValueTree};

#[derive(Clone)]
struct IntTree {
    values: [i32; 2],
    index: usize,
}

impl IntTree {
    fn new(initial: i32, shrunk: i32) -> Self {
        Self {
            values: [initial, shrunk],
            index: 0,
        }
    }
}

impl ValueTree for IntTree {
    type Value = i32;

    fn current(&self) -> &Self::Value {
        &self.values[self.index]
    }

    fn simplify(&mut self) -> bool {
        if self.index + 1 < self.values.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn complicate(&mut self) -> bool {
        if self.index > 0 {
            self.index -= 1;
            self.index > 0
        } else {
            false
        }
    }
}

#[test]
fn vec_never_shrinks_below_min_len() {
    let trees = vec![IntTree::new(3, 1), IntTree::new(4, 1)];
    let mut tree = VecValueTree::from_trees(trees, 2);
    let _ = tree.simplify();
    assert!(tree.current().len() >= 2);
}

#[test]
fn vec_complicate_can_restore_length() {
    let trees = vec![
        IntTree::new(3, 0),
        IntTree::new(4, 0),
        IntTree::new(5, 0),
        IntTree::new(6, 0),
    ];
    let mut tree = VecValueTree::from_trees(trees, 0);
    assert!(tree.simplify(), "expected a length-reduction step");
    let reduced_len = tree.current().len();
    assert!(reduced_len < 4);
    let _ = tree.complicate();
    assert!(
        tree.current().len() >= reduced_len,
        "complicate should not decrease length further"
    );
}
