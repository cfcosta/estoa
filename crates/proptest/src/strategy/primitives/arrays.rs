use std::{array, convert::TryInto};

use crate::strategy::{
    Strategy,
    ValueTree,
    runtime::{Generation, Generator},
};

pub struct ArrayStrategy<S, const N: usize>
where
    S: Strategy,
    S::Value: Clone,
{
    element: S,
}

impl<S, const N: usize> ArrayStrategy<S, N>
where
    S: Strategy,
    S::Value: Clone,
{
    pub fn new(element: S) -> Self {
        Self { element }
    }
}

impl<S, const N: usize> Strategy for ArrayStrategy<S, N>
where
    S: Strategy,
    S::Value: Clone,
{
    type Value = [S::Value; N];
    type Tree = ArrayValueTree<S::Tree, N>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let mut trees = Vec::with_capacity(N);
        for _ in 0..N {
            match self.element.new_tree(generator) {
                Generation::Accepted { value, .. } => trees.push(value),
                Generation::Rejected {
                    iteration, depth, ..
                } => panic!(
                    "array strategy inner component rejected at iteration {iteration}, depth {depth}"
                ),
            }
        }

        let trees: [S::Tree; N] = match trees.try_into() {
            Ok(array) => array,
            Err(_) => {
                panic!("array strategy collected incorrect number of elements")
            }
        };
        let current = array::from_fn(|idx| trees[idx].current().clone());

        generator.accept(ArrayValueTree::new(trees, current))
    }
}

pub struct ArrayValueTree<T, const N: usize>
where
    T: ValueTree,
    T::Value: Clone,
{
    trees: [T; N],
    current: [T::Value; N],
    last_changed: Option<usize>,
}

impl<T, const N: usize> ArrayValueTree<T, N>
where
    T: ValueTree,
    T::Value: Clone,
{
    pub fn new(trees: [T; N], current: [T::Value; N]) -> Self {
        Self {
            trees,
            current,
            last_changed: None,
        }
    }
}

impl<T, const N: usize> ValueTree for ArrayValueTree<T, N>
where
    T: ValueTree,
    T::Value: Clone,
{
    type Value = [T::Value; N];

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        for idx in 0..N {
            if self.trees[idx].simplify() {
                self.current[idx] = self.trees[idx].current().clone();
                self.last_changed = Some(idx);
                return true;
            }
        }
        false
    }

    fn complicate(&mut self) -> bool {
        let Some(idx) = self.last_changed else {
            return false;
        };

        let result = self.trees[idx].complicate();
        self.current[idx] = self.trees[idx].current().clone();
        if result {
            true
        } else {
            self.last_changed = None;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::primitives::IntValueTree;

    #[test]
    fn array_value_tree_shrinks_elements_in_order() {
        let trees =
            [IntValueTree::new(6, vec![1]), IntValueTree::new(4, vec![2])];
        let current = [6, 4];
        let mut tree = ArrayValueTree::new(trees, current);
        assert!(tree.simplify());
        assert_eq!(tree.current()[0], 1);
    }

    #[test]
    fn array_value_tree_complicate_restores_previous() {
        let trees =
            [IntValueTree::new(6, vec![3]), IntValueTree::new(4, vec![2])];
        let current = [6, 4];
        let mut tree = ArrayValueTree::new(trees, current);
        assert!(tree.simplify());
        let _ = tree.complicate();
        assert_eq!(tree.current()[0], 6);
    }
}
