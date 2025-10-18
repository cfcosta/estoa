use rand::Rng;

use crate::strategy::{
    Strategy,
    ValueTree,
    runtime::{Generation, Generator},
};

/// Strategy producing `Option` values from an inner strategy.
pub struct OptionStrategy<S> {
    inner: S,
}

impl<S> OptionStrategy<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Strategy for OptionStrategy<S>
where
    S: Strategy,
    S::Value: Clone,
{
    type Value = Option<S::Value>;
    type Tree = OptionValueTree<S::Tree>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let choose_some = generator.rng.random::<bool>();
        if choose_some {
            match self.inner.new_tree(generator) {
                Generation::Accepted {
                    iteration,
                    depth,
                    value,
                } => {
                    let current = Some(value.current().clone());
                    Generation::Accepted {
                        iteration,
                        depth,
                        value: OptionValueTree::some(value, current),
                    }
                }
                Generation::Rejected {
                    iteration,
                    depth,
                    value,
                } => Generation::Rejected {
                    iteration,
                    depth,
                    value: OptionValueTree::from_inner(value),
                },
            }
        } else {
            generator.accept(OptionValueTree::none())
        }
    }
}

enum OptionState<T>
where
    T: ValueTree,
{
    None,
    Some {
        tree: T,
        tried_none: bool,
        at_none: bool,
    },
}

pub struct OptionValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    state: OptionState<T>,
    current: Option<T::Value>,
}

impl<T> OptionValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    pub fn some(tree: T, current: Option<T::Value>) -> Self {
        Self {
            state: OptionState::Some {
                tree,
                tried_none: false,
                at_none: false,
            },
            current,
        }
    }

    pub fn none() -> Self {
        Self {
            state: OptionState::None,
            current: None,
        }
    }

    fn from_inner(tree: T) -> Self {
        let current = Some(tree.current().clone());
        Self::some(tree, current)
    }
}

impl<T> ValueTree for OptionValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    type Value = Option<T::Value>;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        match &mut self.state {
            OptionState::None => false,
            OptionState::Some {
                tree,
                tried_none,
                at_none,
            } => {
                if !*tried_none {
                    *tried_none = true;
                    *at_none = true;
                    self.current = None;
                    true
                } else if tree.simplify() {
                    *at_none = false;
                    self.current = Some(tree.current().clone());
                    true
                } else {
                    false
                }
            }
        }
    }

    fn complicate(&mut self) -> bool {
        match &mut self.state {
            OptionState::None => false,
            OptionState::Some {
                tree,
                tried_none: _,
                at_none,
            } => {
                if *at_none {
                    *at_none = false;
                    self.current = Some(tree.current().clone());
                    true
                } else if tree.complicate() {
                    self.current = Some(tree.current().clone());
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::primitives::IntValueTree;

    #[test]
    fn option_prefers_none_first() {
        let inner_tree = IntValueTree::new(5, vec![2, 1]);
        let mut tree = OptionValueTree::some(inner_tree, Some(5));
        assert!(tree.simplify());
        assert_eq!(tree.current(), &None);
    }

    #[test]
    fn option_complicate_reverts_to_some() {
        let inner_tree = IntValueTree::new(5, vec![2, 1]);
        let mut tree = OptionValueTree::some(inner_tree, Some(5));
        assert!(tree.simplify());
        assert!(tree.complicate());
        assert_eq!(tree.current(), &Some(5));
    }
}
