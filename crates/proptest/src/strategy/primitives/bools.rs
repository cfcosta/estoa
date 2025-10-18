use rand::Rng;

use crate::{
    strategies::{Generation, Generator},
    strategy::{Strategy, ValueTree},
};

pub struct BoolValueTree {
    current: bool,
    original: bool,
    tried_false: bool,
    can_complicate: bool,
}

impl BoolValueTree {
    pub fn new(value: bool) -> Self {
        Self {
            current: value,
            original: value,
            tried_false: !value,
            can_complicate: false,
        }
    }
}

impl ValueTree for BoolValueTree {
    type Value = bool;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        if self.tried_false {
            return false;
        }

        self.tried_false = true;
        self.can_complicate = self.original;

        if self.original {
            self.current = false;
            true
        } else {
            false
        }
    }

    fn complicate(&mut self) -> bool {
        if !self.can_complicate {
            return false;
        }

        self.current = self.original;
        self.can_complicate = false;
        false
    }
}

#[derive(Default, Clone, Copy)]
pub struct AnyBool;

impl AnyBool {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for AnyBool {
    type Value = bool;
    type Tree = BoolValueTree;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let value = generator.rng.random::<bool>();
        generator.accept(BoolValueTree::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplify_prefers_false() {
        let mut tree = BoolValueTree::new(true);
        assert!(tree.simplify());
        assert!(!tree.simplify());
        assert_eq!(*tree.current(), false);
    }

    #[test]
    fn complicate_reverts_to_original() {
        let mut tree = BoolValueTree::new(true);
        assert!(tree.simplify());
        assert_eq!(*tree.current(), false);
        assert!(!tree.complicate());
        assert_eq!(*tree.current(), true);
        assert!(!tree.simplify());
    }

    #[test]
    fn already_false_is_minimal() {
        let mut tree = BoolValueTree::new(false);
        assert!(!tree.simplify());
        assert!(!tree.complicate());
        assert_eq!(*tree.current(), false);
    }
}
