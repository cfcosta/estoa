use std::{
    collections::{BTreeMap, BTreeSet},
    ops::RangeInclusive,
};

use super::vecs::{build_drop_plan, sample_length};
use crate::strategy::{
    SizeHint,
    Strategy,
    ValueTree,
    runtime::{Generation, Generator, MAX_STRATEGY_ATTEMPTS},
};

#[derive(Clone)]
pub struct BTreeMapStrategy<KS, VS>
where
    KS: Strategy,
    VS: Strategy,
    KS::Value: Clone + Ord,
    VS::Value: Clone,
{
    key: KS,
    value: VS,
    len_range: RangeInclusive<usize>,
}

impl<KS, VS> BTreeMapStrategy<KS, VS>
where
    KS: Strategy,
    VS: Strategy,
    KS::Value: Clone + Ord,
    VS::Value: Clone,
{
    pub fn new<H>(key: KS, value: VS, size_hint: H) -> Self
    where
        H: SizeHint,
    {
        Self {
            key,
            value,
            len_range: size_hint.to_inclusive(),
        }
    }
}

pub struct BTreeMapValueTree<KT, VT>
where
    KT: ValueTree,
    KT::Value: Clone + Ord,
    VT: ValueTree,
    VT::Value: Clone,
{
    entries: Vec<(KT, VT)>,
    keys: Vec<KT::Value>,
    values: Vec<VT::Value>,
    min_len: usize,
    drop_plan: Vec<usize>,
    stage: MapStage,
    history: Vec<MapHistory<KT, VT>>,
    current: BTreeMap<KT::Value, VT::Value>,
}

#[derive(Clone, Copy)]
enum MapStage {
    Length { chunk_index: usize, offset: usize },
    Keys { index: usize },
    Values { index: usize },
}

enum MapHistory<KT, VT>
where
    KT: ValueTree,
    VT: ValueTree,
{
    RemovedChunk {
        index: usize,
        chunk_index: usize,
        entries: Vec<(KT, VT)>,
        keys: Vec<KT::Value>,
        values: Vec<VT::Value>,
    },
    Key {
        index: usize,
    },
    Value {
        index: usize,
    },
}

impl<KT, VT> BTreeMapValueTree<KT, VT>
where
    KT: ValueTree,
    KT::Value: Clone + Ord,
    VT: ValueTree,
    VT::Value: Clone,
{
    pub fn from_entries(
        entries: Vec<(KT, VT)>,
        keys: Vec<KT::Value>,
        values: Vec<VT::Value>,
        min_len: usize,
    ) -> Self {
        let drop_plan = build_drop_plan(entries.len());
        let stage = if drop_plan.is_empty() {
            MapStage::Keys { index: 0 }
        } else {
            MapStage::Length {
                chunk_index: 0,
                offset: 0,
            }
        };

        let mut tree = Self {
            entries,
            keys,
            values,
            min_len,
            drop_plan,
            stage,
            history: Vec::new(),
            current: BTreeMap::new(),
        };

        tree.rebuild_current();
        tree
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn rebuild_current(&mut self) {
        self.current.clear();
        for (key, value) in
            self.keys.iter().cloned().zip(self.values.iter().cloned())
        {
            self.current.insert(key, value);
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

            self.stage = MapStage::Length {
                chunk_index,
                offset,
            };
            return Some((chunk_index, offset, chunk_size));
        }

        self.stage = MapStage::Keys { index: 0 };
        None
    }

    fn key_duplicate(&self, index: usize, candidate: &KT::Value) -> bool {
        self.keys
            .iter()
            .enumerate()
            .any(|(i, key)| i != index && key == candidate)
    }
}

impl<KS, VS> Strategy for BTreeMapStrategy<KS, VS>
where
    KS: Strategy,
    VS: Strategy,
    KS::Value: Clone + Ord,
    VS::Value: Clone,
{
    type Value = BTreeMap<KS::Value, VS::Value>;
    type Tree = BTreeMapValueTree<KS::Tree, VS::Tree>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let target_len = sample_length(&mut generator.rng, &self.len_range);
        let min_len = *self.len_range.start();
        let mut entries = Vec::with_capacity(target_len);
        let mut keys = Vec::with_capacity(target_len);
        let mut values = Vec::with_capacity(target_len);
        let mut seen = BTreeSet::new();

        let mut attempts_remaining = MAX_STRATEGY_ATTEMPTS * target_len.max(1);

        while entries.len() < target_len && attempts_remaining > 0 {
            attempts_remaining -= 1;

            let key_tree = match self.key.new_tree(generator) {
                Generation::Accepted { value, .. } => value,
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    let tree = BTreeMapValueTree::from_entries(
                        entries, keys, values, min_len,
                    );
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: tree,
                    };
                }
            };

            let candidate_key = key_tree.current().clone();
            if !seen.insert(candidate_key.clone()) {
                continue;
            }

            let value_tree = match self.value.new_tree(generator) {
                Generation::Accepted { value, .. } => value,
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    let tree = BTreeMapValueTree::from_entries(
                        entries, keys, values, min_len,
                    );
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: tree,
                    };
                }
            };

            keys.push(candidate_key);
            values.push(value_tree.current().clone());
            entries.push((key_tree, value_tree));
        }

        generator.accept(BTreeMapValueTree::from_entries(
            entries, keys, values, min_len,
        ))
    }
}

impl<KT, VT> ValueTree for BTreeMapValueTree<KT, VT>
where
    KT: ValueTree,
    KT::Value: Clone + Ord,
    VT: ValueTree,
    VT::Value: Clone,
{
    type Value = BTreeMap<KT::Value, VT::Value>;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        loop {
            match self.stage {
                MapStage::Length {
                    chunk_index,
                    offset,
                } => {
                    let Some((ci, off, chunk_size)) =
                        self.seek_length_from(chunk_index, offset)
                    else {
                        continue;
                    };

                    let entries: Vec<(KT, VT)> =
                        self.entries.drain(off..off + chunk_size).collect();
                    let keys: Vec<KT::Value> =
                        self.keys.drain(off..off + chunk_size).collect();
                    let values: Vec<VT::Value> =
                        self.values.drain(off..off + chunk_size).collect();
                    self.rebuild_current();
                    self.history.push(MapHistory::RemovedChunk {
                        index: off,
                        chunk_index: ci,
                        entries,
                        keys,
                        values,
                    });
                    return true;
                }
                MapStage::Keys { index } => {
                    if index >= self.len() {
                        self.stage = MapStage::Values { index: 0 };
                        continue;
                    }

                    if self.entries[index].0.simplify() {
                        let candidate = self.entries[index].0.current().clone();
                        if self.key_duplicate(index, &candidate) {
                            if !self.entries[index].0.complicate() {
                                self.stage =
                                    MapStage::Keys { index: index + 1 };
                            }
                            continue;
                        }

                        self.keys[index] = candidate;
                        self.rebuild_current();
                        self.history.push(MapHistory::Key { index });
                        return true;
                    } else {
                        self.stage = MapStage::Keys { index: index + 1 };
                    }
                }
                MapStage::Values { index } => {
                    if index >= self.len() {
                        return false;
                    }

                    if self.entries[index].1.simplify() {
                        self.values[index] =
                            self.entries[index].1.current().clone();
                        self.rebuild_current();
                        self.history.push(MapHistory::Value { index });
                        return true;
                    } else {
                        self.stage = MapStage::Values { index: index + 1 };
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
            MapHistory::RemovedChunk {
                index,
                chunk_index,
                entries,
                keys,
                values,
            } => {
                self.entries.splice(index..index, entries);
                self.keys.splice(index..index, keys);
                self.values.splice(index..index, values);
                self.rebuild_current();
                match self.seek_length_from(chunk_index, index + 1) {
                    Some(_) => true,
                    None => {
                        self.stage = MapStage::Keys { index: 0 };
                        !self.entries.is_empty()
                    }
                }
            }
            MapHistory::Key { index } => {
                if self.entries[index].0.complicate() {
                    self.keys[index] = self.entries[index].0.current().clone();
                    self.rebuild_current();
                    self.history.push(MapHistory::Key { index });
                    true
                } else {
                    self.keys[index] = self.entries[index].0.current().clone();
                    self.rebuild_current();
                    if index + 1 < self.len() {
                        self.stage = MapStage::Keys { index: index + 1 };
                        true
                    } else {
                        self.stage = MapStage::Values { index: 0 };
                        !self.entries.is_empty()
                    }
                }
            }
            MapHistory::Value { index } => {
                if self.entries[index].1.complicate() {
                    self.values[index] =
                        self.entries[index].1.current().clone();
                    self.rebuild_current();
                    self.history.push(MapHistory::Value { index });
                    true
                } else {
                    self.values[index] =
                        self.entries[index].1.current().clone();
                    self.rebuild_current();
                    if index + 1 < self.len() {
                        self.stage = MapStage::Values { index: index + 1 };
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
    fn btree_map_shrinking_preserves_keys() {
        let entries = vec![
            (make_tree(7, 1), IntValueTree::new(10, vec![0])),
            (make_tree(3, 1), IntValueTree::new(5, vec![4])),
        ];

        let keys = entries
            .iter()
            .map(|(k, _): &(IntValueTree<i32>, IntValueTree<i32>)| *k.current())
            .collect::<Vec<_>>();
        let values = entries
            .iter()
            .map(|(_, v): &(IntValueTree<i32>, IntValueTree<i32>)| *v.current())
            .collect::<Vec<_>>();
        let mut tree =
            BTreeMapValueTree::from_entries(entries, keys, values, 2);

        assert!(tree.simplify());
        let current = tree.current();
        assert_eq!(current.len(), 2);
        let mut prev: Option<i32> = None;
        for key in current.keys().copied() {
            if let Some(previous) = prev {
                assert_ne!(previous, key);
            }
            prev = Some(key);
        }
    }

    #[test]
    fn btree_map_strategy_honours_range() {
        let mut strategy = BTreeMapStrategy::new(
            AnyI32::default(),
            AnyI32::default(),
            1usize..=3usize,
        );
        let mut generator =
            Generator::build_with_limit(crate::rng(), usize::MAX);
        let len = match strategy.new_tree(&mut generator) {
            Generation::Accepted { value, .. } => value.current().len(),
            Generation::Rejected { .. } => panic!("unexpected rejection"),
        };
        assert!((1..=3).contains(&len));
    }
}
