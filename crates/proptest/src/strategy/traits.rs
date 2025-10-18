use rand::{CryptoRng, RngCore};

use crate::strategy::runtime::{Generation, Generator};

/// A shrinkable search space for values produced by a [`Strategy`].
pub trait ValueTree {
    type Value;

    /// Borrow the current value represented by this tree.
    fn current(&self) -> &Self::Value;

    /// Attempt to move to a strictly simpler candidate.
    ///
    /// Returns `true` when the tree advanced to a new candidate, regardless of
    /// whether the property still fails with that candidate.
    fn simplify(&mut self) -> bool;

    /// Backtrack after a failed simplify attempt.
    ///
    /// Returns `true` when there are more alternatives remaining from the
    /// current node in the tree.
    fn complicate(&mut self) -> bool;
}

/// A generator of [`ValueTree`] instances.
pub trait Strategy {
    type Value;
    type Tree: ValueTree<Value = Self::Value>;

    fn new_tree<R: RngCore + CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree>;
}
