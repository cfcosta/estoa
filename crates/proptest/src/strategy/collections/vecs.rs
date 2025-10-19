use std::{
    collections::{BinaryHeap, VecDeque},
    ops::RangeInclusive,
};

use super::super::primitives::AnyUsize;
use crate::strategy::{
    SizeHint,
    Strategy,
    ValueTree,
    runtime::{Generation, Generator},
};

pub(crate) fn build_drop_plan(len: usize) -> Vec<usize> {
    let mut plan = Vec::new();
    let mut size = len / 2;

    while size > 0 {
        plan.push(size);
        size /= 2;
    }

    if !plan.contains(&1) && len > 0 {
        plan.push(1);
    }

    plan
}

pub(crate) fn sample_length<R: rand::RngCore + rand::CryptoRng>(
    rng: &mut R,
    range: &RangeInclusive<usize>,
) -> usize {
    AnyUsize::sample(rng, range.clone())
}

#[derive(Clone)]
pub struct VecStrategy<S>
where
    S: Strategy,
    S::Value: Clone,
{
    element: S,
    len_range: RangeInclusive<usize>,
}

impl<S> VecStrategy<S>
where
    S: Strategy,
    S::Value: Clone,
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

impl<S> Strategy for VecStrategy<S>
where
    S: Strategy,
    S::Value: Clone,
{
    type Value = Vec<S::Value>;
    type Tree = VecValueTree<S::Tree>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let len = sample_length(&mut generator.rng, &self.len_range);
        let min_len = *self.len_range.start();
        let mut trees = Vec::with_capacity(len);

        for _ in 0..len {
            match self.element.new_tree(generator) {
                Generation::Accepted { value, .. } => trees.push(value),
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: VecValueTree::from_trees(trees, min_len),
                    };
                }
            }
        }

        generator.accept(VecValueTree::from_trees(trees, min_len))
    }
}

#[derive(Clone, Copy)]
enum Stage {
    Length { chunk_index: usize, offset: usize },
    Elements { index: usize },
}

enum History<T> {
    RemovedChunk {
        index: usize,
        chunk_index: usize,
        chunk: Vec<T>,
    },
    Element {
        index: usize,
    },
}

pub struct VecValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    elements: Vec<T>,
    current: Vec<T::Value>,
    min_len: usize,
    drop_plan: Vec<usize>,
    stage: Stage,
    history: Vec<History<T>>,
}

impl<T> VecValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    pub fn from_trees(elements: Vec<T>, min_len: usize) -> Self {
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
            current: Vec::new(),
            min_len,
            drop_plan,
            stage,
            history: Vec::new(),
        };

        tree.sync_current();
        tree
    }

    fn sync_current(&mut self) {
        self.current = self
            .elements
            .iter()
            .map(|element| element.current().clone())
            .collect();
    }

    fn len(&self) -> usize {
        self.elements.len()
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
}

impl<T> ValueTree for VecValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    type Value = Vec<T::Value>;

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

                    let removed: Vec<T> =
                        self.elements.drain(off..off + chunk_size).collect();
                    self.current.drain(off..off + chunk_size).count();
                    self.history.push(History::RemovedChunk {
                        index: off,
                        chunk_index: ci,
                        chunk: removed,
                    });
                    return true;
                }
                Stage::Elements { index } => {
                    if index >= self.len() {
                        return false;
                    }

                    if self.elements[index].simplify() {
                        self.current[index] =
                            self.elements[index].current().clone();
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
                chunk,
            } => {
                let values: Vec<T::Value> =
                    chunk.iter().map(|tree| tree.current().clone()).collect();
                self.elements.splice(index..index, chunk);
                self.current.splice(index..index, values);

                match self.seek_length_from(chunk_index, index + 1) {
                    Some(_) => true,
                    None => !self.current.is_empty(),
                }
            }
            History::Element { index } => {
                if self.elements[index].complicate() {
                    self.current[index] =
                        self.elements[index].current().clone();
                    self.history.push(History::Element { index });
                    true
                } else {
                    self.current[index] =
                        self.elements[index].current().clone();
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

pub struct VecDequeStrategy<S>
where
    S: Strategy,
    S::Value: Clone,
{
    inner: VecStrategy<S>,
}

impl<S> VecDequeStrategy<S>
where
    S: Strategy,
    S::Value: Clone,
{
    pub fn new<H>(element: S, size_hint: H) -> Self
    where
        H: SizeHint,
    {
        Self {
            inner: VecStrategy::new(element, size_hint),
        }
    }
}

pub struct VecDequeValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    inner: VecValueTree<T>,
    current: VecDeque<T::Value>,
}

impl<T> VecDequeValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    fn new(inner: VecValueTree<T>) -> Self {
        let mut tree = Self {
            inner,
            current: VecDeque::new(),
        };
        tree.sync_current();
        tree
    }

    fn sync_current(&mut self) {
        self.current = VecDeque::from(self.inner.current().clone());
    }
}

impl<S> Strategy for VecDequeStrategy<S>
where
    S: Strategy,
    S::Value: Clone,
{
    type Value = VecDeque<S::Value>;
    type Tree = VecDequeValueTree<S::Tree>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        self.inner.new_tree(generator).map(VecDequeValueTree::new)
    }
}

impl<T> ValueTree for VecDequeValueTree<T>
where
    T: ValueTree,
    T::Value: Clone,
{
    type Value = VecDeque<T::Value>;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        if self.inner.simplify() {
            self.sync_current();
            true
        } else {
            false
        }
    }

    fn complicate(&mut self) -> bool {
        if self.inner.complicate() {
            self.sync_current();
            true
        } else {
            false
        }
    }
}

pub struct BinaryHeapStrategy<S>
where
    S: Strategy,
    S::Value: Clone + Ord,
{
    inner: VecStrategy<S>,
}

impl<S> BinaryHeapStrategy<S>
where
    S: Strategy,
    S::Value: Clone + Ord,
{
    pub fn new<H>(element: S, size_hint: H) -> Self
    where
        H: SizeHint,
    {
        Self {
            inner: VecStrategy::new(element, size_hint),
        }
    }
}

pub struct BinaryHeapValueTree<T>
where
    T: ValueTree,
    T::Value: Clone + Ord,
{
    inner: VecValueTree<T>,
    current: BinaryHeap<T::Value>,
}

impl<T> BinaryHeapValueTree<T>
where
    T: ValueTree,
    T::Value: Clone + Ord,
{
    fn new(inner: VecValueTree<T>) -> Self {
        let mut tree = Self {
            inner,
            current: BinaryHeap::new(),
        };
        tree.sync_current();
        tree
    }

    fn sync_current(&mut self) {
        self.current = BinaryHeap::from(self.inner.current().clone());
    }
}

impl<S> Strategy for BinaryHeapStrategy<S>
where
    S: Strategy,
    S::Value: Clone + Ord,
{
    type Value = BinaryHeap<S::Value>;
    type Tree = BinaryHeapValueTree<S::Tree>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        self.inner.new_tree(generator).map(BinaryHeapValueTree::new)
    }
}

impl<T> ValueTree for BinaryHeapValueTree<T>
where
    T: ValueTree,
    T::Value: Clone + Ord,
{
    type Value = BinaryHeap<T::Value>;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        if self.inner.simplify() {
            self.sync_current();
            true
        } else {
            false
        }
    }

    fn complicate(&mut self) -> bool {
        if self.inner.complicate() {
            self.sync_current();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::{AnyI32, ValueTree, runtime::Generator};

    #[test]
    fn vec_drop_plan_halves() {
        assert_eq!(build_drop_plan(8), vec![4, 2, 1]);
    }

    #[test]
    fn vec_shrinks_length_first() {
        let trees = vec![IntTree::new(3), IntTree::new(2), IntTree::new(1)];

        let mut tree = VecValueTree::from_trees(trees, 0);
        assert!(tree.simplify());
        assert_eq!(tree.current().len(), 2);
    }

    struct IntTree {
        values: Vec<i32>,
        index: usize,
    }

    impl IntTree {
        fn new(value: i32) -> Self {
            Self {
                values: vec![value, 0],
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
            if self.index == 0 {
                false
            } else {
                self.index -= 1;
                self.index > 0
            }
        }
    }

    #[test]
    fn vec_shrinks_elements_after_length() {
        let trees = vec![IntTree::new(5), IntTree::new(9)];
        let mut tree = VecValueTree::from_trees(trees, 1);

        assert!(tree.simplify());
        assert_eq!(tree.current(), &vec![9]);

        assert!(tree.simplify());
        assert_eq!(tree.current(), &vec![0]);
    }

    #[test]
    fn vec_deque_mirrors_vec_shrinking() {
        let trees = vec![IntTree::new(4), IntTree::new(3), IntTree::new(2)];
        let inner = VecValueTree::from_trees(trees, 0);
        let mut tree = VecDequeValueTree::new(inner);

        assert_eq!(tree.current().len(), 3);
        assert!(tree.simplify());
        assert_eq!(tree.current().len(), 2);
        assert!(tree.simplify());
        assert_eq!(tree.current().len(), 1);
    }

    #[test]
    fn binary_heap_preserves_heap_property() {
        let trees = vec![IntTree::new(7), IntTree::new(3), IntTree::new(5)];
        let inner = VecValueTree::from_trees(trees, 1);
        let mut tree = BinaryHeapValueTree::new(inner);

        assert_eq!(tree.current().len(), 3);
        assert_eq!(tree.current().peek(), Some(&7));

        assert!(tree.simplify());
        assert_eq!(tree.current().len(), 2);
        assert_eq!(tree.current().peek(), Some(&5));

        assert!(tree.simplify());
        assert_eq!(tree.current().len(), 1);
        assert_eq!(tree.current().peek(), Some(&5));
    }

    #[test]
    fn vec_deque_strategy_yields_len_in_range() {
        let mut strategy =
            VecDequeStrategy::new(AnyI32::default(), 1usize..=3usize);
        let mut generator = Generator::build(crate::rng());
        let tree = match strategy.new_tree(&mut generator) {
            Generation::Accepted { value, .. } => value,
            Generation::Rejected { .. } => panic!("unexpected rejection"),
        };
        let len = tree.current().len();
        assert!((1..=3).contains(&len));
    }

    #[test]
    fn binary_heap_strategy_yields_len_in_range() {
        let mut strategy =
            BinaryHeapStrategy::new(AnyI32::default(), 1usize..=3usize);
        let mut generator = Generator::build(crate::rng());
        let tree = match strategy.new_tree(&mut generator) {
            Generation::Accepted { value, .. } => value,
            Generation::Rejected { .. } => panic!("unexpected rejection"),
        };
        let len = tree.current().len();
        assert!((1..=3).contains(&len));
    }

    #[test]
    fn vec_strategy_builds_length_in_range() {
        let mut strategy = VecStrategy::new(AnyI32::default(), 2usize..=4usize);
        let mut generator = Generator::build(crate::rng());
        let tree = match strategy.new_tree(&mut generator) {
            Generation::Accepted { value, .. } => value,
            Generation::Rejected { .. } => panic!("unexpected rejection"),
        };
        let len = tree.current().len();
        assert!((2..=4).contains(&len), "len out of range");
    }
}
