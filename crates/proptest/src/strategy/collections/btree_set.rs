use std::{collections::BTreeSet, ops::RangeInclusive};

use super::vecs::{build_drop_plan, sample_length};
use crate::strategy::{
    SizeHint,
    Strategy,
    ValueTree,
    runtime::{Generation, Generator, MAX_STRATEGY_ATTEMPTS},
};

#[derive(Clone)]
pub struct BTreeSetStrategy<S>
where
    S: Strategy,
    S::Value: Clone + Ord,
{
    element: S,
    len_range: RangeInclusive<usize>,
}

impl<S> BTreeSetStrategy<S>
where
    S: Strategy,
    S::Value: Clone + Ord,
{
    pub fn new<H>(element: S, size_hint: H) -> Self
    where
        H: SizeHint,
    {
        Self {
            element,
            len_range: size_hint.to_inclusive(),
        }
    }
}

pub struct BTreeSetValueTree<T>
where
    T: ValueTree,
    T::Value: Clone + Ord,
{
    elements: Vec<T>,
    raw_values: Vec<T::Value>,
    min_len: usize,
    drop_plan: Vec<usize>,
    stage: Stage,
    history: Vec<History<T, T::Value>>,
    current: BTreeSet<T::Value>,
}

#[derive(Clone, Copy)]
enum Stage {
    Length { chunk_index: usize, offset: usize },
    Elements { index: usize },
}

enum History<T, V> {
    RemovedChunk {
        index: usize,
        chunk_index: usize,
        trees: Vec<T>,
        values: Vec<V>,
    },
    Element {
        index: usize,
    },
}

impl<S> Strategy for BTreeSetStrategy<S>
where
    S: Strategy,
    S::Value: Clone + Ord,
{
    type Value = BTreeSet<S::Value>;
    type Tree = BTreeSetValueTree<S::Tree>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let target_len = sample_length(&mut generator.rng, &self.len_range);
        let min_len = *self.len_range.start();
        let mut elements = Vec::with_capacity(target_len);
        let mut values = Vec::with_capacity(target_len);
        let mut seen = BTreeSet::new();

        let mut attempts_remaining = MAX_STRATEGY_ATTEMPTS * target_len.max(1);

        while elements.len() < target_len && attempts_remaining > 0 {
            attempts_remaining -= 1;

            match self.element.new_tree(generator) {
                Generation::Accepted { value, .. } => {
                    let candidate = value.current().clone();
                    if seen.insert(candidate.clone()) {
                        elements.push(value);
                        values.push(candidate);
                    }
                }
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    let tree = BTreeSetValueTree::from_elements(
                        elements, values, min_len,
                    );
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: tree,
                    };
                }
            }
        }

        generator
            .accept(BTreeSetValueTree::from_elements(elements, values, min_len))
    }
}

impl<T> BTreeSetValueTree<T>
where
    T: ValueTree,
    T::Value: Clone + Ord,
{
    pub fn from_elements(
        elements: Vec<T>,
        raw_values: Vec<T::Value>,
        min_len: usize,
    ) -> Self {
        let drop_plan = build_drop_plan(elements.len());
        let stage = if drop_plan.is_empty() {
            Stage::Elements { index: 0 }
        } else {
            Stage::Length {
                chunk_index: 0,
                offset: 0,
            }
        };

        let mut tree = Self {
            elements,
            raw_values,
            min_len,
            drop_plan,
            stage,
            history: Vec::new(),
            current: BTreeSet::new(),
        };

        tree.rebuild_current();
        tree
    }

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn rebuild_current(&mut self) {
        self.current.clear();
        for value in &self.raw_values {
            self.current.insert(value.clone());
        }
    }

    fn seek_length_from(
        &mut self,
        mut chunk_index: usize,
        mut offset: usize,
    ) -> Option<(usize, usize, usize)> {
        while chunk_index < self.drop_plan.len() {
            let chunk_size = self.drop_plan[chunk_index];

            if chunk_size == 0
                || self.len() <= self.min_len
                || chunk_size > self.len()
                || self.len().saturating_sub(chunk_size) < self.min_len
            {
                chunk_index += 1;
                offset = 0;
                continue;
            }

            if offset + chunk_size > self.len() {
                chunk_index += 1;
                offset = 0;
                continue;
            }

            self.stage = Stage::Length {
                chunk_index,
                offset,
            };
            return Some((chunk_index, offset, chunk_size));
        }

        self.stage = Stage::Elements { index: 0 };
        None
    }

    fn element_duplicate(&self, index: usize, candidate: &T::Value) -> bool {
        self.raw_values
            .iter()
            .enumerate()
            .any(|(i, value)| i != index && value == candidate)
    }
}

impl<T> ValueTree for BTreeSetValueTree<T>
where
    T: ValueTree,
    T::Value: Clone + Ord,
{
    type Value = BTreeSet<T::Value>;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        loop {
            match self.stage {
                Stage::Length {
                    chunk_index,
                    offset,
                } => {
                    let Some((ci, off, chunk_size)) =
                        self.seek_length_from(chunk_index, offset)
                    else {
                        continue;
                    };

                    let trees: Vec<T> =
                        self.elements.drain(off..off + chunk_size).collect();
                    let values: Vec<T::Value> =
                        self.raw_values.drain(off..off + chunk_size).collect();
                    self.rebuild_current();
                    self.history.push(History::RemovedChunk {
                        index: off,
                        chunk_index: ci,
                        trees,
                        values,
                    });
                    return true;
                }
                Stage::Elements { index } => {
                    if index >= self.len() {
                        return false;
                    }

                    if self.elements[index].simplify() {
                        let candidate = self.elements[index].current().clone();
                        if self.element_duplicate(index, &candidate) {
                            if !self.elements[index].complicate() {
                                self.stage =
                                    Stage::Elements { index: index + 1 };
                            }
                            continue;
                        }

                        self.raw_values[index] = candidate;
                        self.rebuild_current();
                        self.history.push(History::Element { index });
                        return true;
                    } else {
                        self.stage = Stage::Elements { index: index + 1 };
                    }
                }
            }
        }
    }

    fn complicate(&mut self) -> bool {
        let Some(entry) = self.history.pop() else {
            return false;
        };

        match entry {
            History::RemovedChunk {
                index,
                chunk_index,
                trees,
                values,
            } => {
                self.elements.splice(index..index, trees);
                self.raw_values.splice(index..index, values);
                self.rebuild_current();
                match self.seek_length_from(chunk_index, index + 1) {
                    Some(_) => true,
                    None => !self.elements.is_empty(),
                }
            }
            History::Element { index } => {
                if self.elements[index].complicate() {
                    self.raw_values[index] =
                        self.elements[index].current().clone();
                    self.rebuild_current();
                    self.history.push(History::Element { index });
                    true
                } else {
                    self.raw_values[index] =
                        self.elements[index].current().clone();
                    self.rebuild_current();
                    if index + 1 < self.len() {
                        self.stage = Stage::Elements { index: index + 1 };
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::{
        ValueTree,
        primitives::{AnyI32, IntValueTree},
        runtime::Generator,
    };

    fn make_tree(value: i32, shrink_to: i32) -> IntValueTree<i32> {
        IntValueTree::new(value, vec![shrink_to])
    }

    #[test]
    fn btree_set_shrinking_preserves_uniqueness() {
        let trees = vec![make_tree(5, 3), make_tree(2, 3)];
        let values = trees
            .iter()
            .map(|tree: &IntValueTree<i32>| *tree.current())
            .collect::<Vec<_>>();
        let mut tree = BTreeSetValueTree::from_elements(trees, values, 2);

        assert!(tree.simplify());
        let current = tree.current();
        assert_eq!(current.len(), 2);
        assert!(current.contains(&3));
        assert!(current.contains(&2));
    }

    #[test]
    fn btree_set_strategy_honours_range() {
        let mut strategy =
            BTreeSetStrategy::new(AnyI32::default(), 1usize..=3usize);
        let mut generator =
            Generator::build_with_limit(crate::rng(), usize::MAX);
        let len = match strategy.new_tree(&mut generator) {
            Generation::Accepted { value, .. } => value.current().len(),
            Generation::Rejected { .. } => panic!("unexpected rejection"),
        };
        assert!((1..=3).contains(&len));
    }
}
