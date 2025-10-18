use paste::paste;

use crate::{
    strategies::{Generation, Generator},
    strategy::{Strategy, ValueTree},
};

macro_rules! tuple_impl {
    ($($len:literal => { $($idx:tt : $field:ident),+ } ),+ $(,)?) => {
        paste! {
            $(
                pub struct [<TupleValueTree $len>]<$($field),+>
                where
                    $( $field: ValueTree, $field::Value: Clone ),+
                {
                    trees: ($($field,)+),
                    current: ($($field::Value,)+),
                    last_changed: Option<usize>,
                }

                impl<$($field),+> [<TupleValueTree $len>]<$($field),+>
                where
                    $( $field: ValueTree, $field::Value: Clone ),+
                {
                    fn new(trees: ($($field,)+)) -> Self {
                        let current = (
                            $( trees.$idx.current().clone(), )+
                        );
                        Self {
                            trees,
                            current,
                            last_changed: None,
                        }
                    }

                    fn update_field(&mut self, index: usize) {
                        match index {
                            $(
                                $idx => {
                                    self.current.$idx = self.trees.$idx.current().clone();
                                }
                            )+
                            _ => unreachable!(),
                        }
                    }
                }

                impl<$($field),+> ValueTree for [<TupleValueTree $len>]<$($field),+>
                where
                    $( $field: ValueTree, $field::Value: Clone ),+
                {
                    type Value = ($($field::Value,)+);

                    fn current(&self) -> &Self::Value {
                        &self.current
                    }

                    fn simplify(&mut self) -> bool {
                        $(
                            if self.trees.$idx.simplify() {
                                self.update_field($idx);
                                self.last_changed = Some($idx);
                                return true;
                            }
                        )+
                        false
                    }

                    fn complicate(&mut self) -> bool {
                        let Some(idx) = self.last_changed else {
                            return false;
                        };

                        match idx {
                            $(
                                $idx => {
                                    let result = self.trees.$idx.complicate();
                                    self.update_field($idx);
                                    if result {
                                        true
                                    } else {
                                        self.last_changed = None;
                                        false
                                    }
                                }
                            )+
                            _ => unreachable!(),
                        }
                    }
                }

                impl<$($field),+> Strategy for ($($field,)+)
                where
                    $( $field: Strategy, $field::Value: Clone ),+
                {
                    type Value = ($($field::Value,)+);
                    type Tree = [<TupleValueTree $len>]<$($field::Tree,)+>;

                    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
                        &mut self,
                        generator: &mut Generator<R>,
                    ) -> Generation<Self::Tree> {
                        let trees = (
                            $(
                                match self.$idx.new_tree(generator) {
                                    Generation::Accepted { value, .. } => value,
                                    Generation::Rejected { iteration, depth, .. } => {
                                        panic!(
                                            "tuple component {} rejected at iteration {}, depth {}",
                                            $idx,
                                            iteration,
                                            depth,
                                        );
                                    }
                                },
                            )+
                        );

                        generator.accept([<TupleValueTree $len>]::new(trees))
                    }
                }
            )+
        }
    };
}

tuple_impl! {
    1 => { 0: A },
    2 => { 0: A, 1: B },
    3 => { 0: A, 1: B, 2: C },
    4 => { 0: A, 1: B, 2: C, 3: D },
    5 => { 0: A, 1: B, 2: C, 3: D, 4: E },
    6 => { 0: A, 1: B, 2: C, 3: D, 4: E, 5: F },
    7 => { 0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G },
    8 => { 0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H },
    9 => { 0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I },
    10 => { 0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J },
    11 => { 0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K },
    12 => { 0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::primitives::integers::IntValueTree;

    #[test]
    fn tuple_value_tree_prefers_first_field() {
        let mut tree = TupleValueTree2::new((
            IntValueTree::new(5, vec![1]),
            IntValueTree::new(7, vec![3]),
        ));
        assert!(tree.simplify());
        assert_eq!(tree.current().0, 1);
    }

    #[test]
    fn tuple_value_tree_complicate_restores_field() {
        let mut tree = TupleValueTree2::new((
            IntValueTree::new(5, vec![1]),
            IntValueTree::new(7, vec![3]),
        ));
        assert!(tree.simplify());
        let _ = tree.complicate();
        assert_eq!(tree.current().0, 5);
    }
}
