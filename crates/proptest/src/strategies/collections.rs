use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    hash::Hash,
};

use super::{DefaultGenerator, Generation, MAX_STRATEGY_ATTEMPTS, SizeHint};

pub fn vec<T, S, H>(
    mut strategy: S,
    hint: H,
) -> impl FnMut(&mut DefaultGenerator) -> Generation<Vec<T>>
where
    S: FnMut(&mut DefaultGenerator) -> Generation<T>,
    H: SizeHint,
{
    move |generator: &mut DefaultGenerator| {
        let len = hint.pick(&mut generator.rng);
        let mut values = Vec::with_capacity(len);

        for _ in 0..len {
            match strategy(generator) {
                Generation::Accepted { value, .. } => values.push(value),
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: values,
                    };
                }
            }
        }

        generator.accept(values)
    }
}

pub fn vec_deque<T, S, H>(
    mut strategy: S,
    hint: H,
) -> impl FnMut(&mut DefaultGenerator) -> Generation<VecDeque<T>>
where
    S: FnMut(&mut DefaultGenerator) -> Generation<T>,
    H: SizeHint,
{
    move |generator: &mut DefaultGenerator| {
        let len = hint.pick(&mut generator.rng);
        let mut values = VecDeque::with_capacity(len);

        for _ in 0..len {
            match strategy(generator) {
                Generation::Accepted { value, .. } => values.push_back(value),
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: values,
                    };
                }
            }
        }

        generator.accept(values)
    }
}

pub fn binary_heap<T, S, H>(
    mut strategy: S,
    hint: H,
) -> impl FnMut(&mut DefaultGenerator) -> Generation<BinaryHeap<T>>
where
    T: Ord,
    S: FnMut(&mut DefaultGenerator) -> Generation<T>,
    H: SizeHint,
{
    move |generator: &mut DefaultGenerator| {
        let len = hint.pick(&mut generator.rng);
        let mut heap = BinaryHeap::with_capacity(len);

        for _ in 0..len {
            match strategy(generator) {
                Generation::Accepted { value, .. } => heap.push(value),
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: heap,
                    };
                }
            }
        }

        generator.accept(heap)
    }
}

pub fn hash_set<T, S, H>(
    mut strategy: S,
    hint: H,
) -> impl FnMut(&mut DefaultGenerator) -> Generation<HashSet<T>>
where
    T: Eq + Hash,
    S: FnMut(&mut DefaultGenerator) -> Generation<T>,
    H: SizeHint,
{
    move |generator: &mut DefaultGenerator| {
        let target_len = hint.pick(&mut generator.rng);
        let mut set = HashSet::with_capacity(target_len);

        if target_len == 0 {
            return generator.accept(set);
        }

        let mut attempts_remaining = MAX_STRATEGY_ATTEMPTS * target_len.max(1);

        while set.len() < target_len {
            if attempts_remaining == 0 {
                break;
            }

            attempts_remaining -= 1;

            match strategy(generator) {
                Generation::Accepted { value, .. } => {
                    set.insert(value);
                }
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: set,
                    };
                }
            }
        }

        generator.accept(set)
    }
}

pub fn btree_set<T, S, H>(
    mut strategy: S,
    hint: H,
) -> impl FnMut(&mut DefaultGenerator) -> Generation<BTreeSet<T>>
where
    T: Ord,
    S: FnMut(&mut DefaultGenerator) -> Generation<T>,
    H: SizeHint,
{
    move |generator: &mut DefaultGenerator| {
        let target_len = hint.pick(&mut generator.rng);
        let mut set = BTreeSet::new();

        if target_len == 0 {
            return generator.accept(set);
        }

        let mut attempts_remaining = MAX_STRATEGY_ATTEMPTS * target_len.max(1);

        while set.len() < target_len {
            if attempts_remaining == 0 {
                break;
            }
            attempts_remaining -= 1;

            match strategy(generator) {
                Generation::Accepted { value, .. } => {
                    set.insert(value);
                }
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: set,
                    };
                }
            }
        }

        generator.accept(set)
    }
}

pub fn hash_map<K, V, KS, VS, H>(
    mut key_strategy: KS,
    mut value_strategy: VS,
    hint: H,
) -> impl FnMut(&mut DefaultGenerator) -> Generation<HashMap<K, V>>
where
    K: Eq + Hash,
    KS: FnMut(&mut DefaultGenerator) -> Generation<K>,
    VS: FnMut(&mut DefaultGenerator) -> Generation<V>,
    H: SizeHint,
{
    move |generator: &mut DefaultGenerator| {
        let target_len = hint.pick(&mut generator.rng);
        let mut map = HashMap::with_capacity(target_len);

        if target_len == 0 {
            return generator.accept(map);
        }

        let mut attempts_remaining = MAX_STRATEGY_ATTEMPTS * target_len.max(1);

        while map.len() < target_len {
            if attempts_remaining == 0 {
                break;
            }
            attempts_remaining -= 1;

            let key = match key_strategy(generator) {
                Generation::Accepted { value, .. } => value,
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: map,
                    };
                }
            };

            let value = match value_strategy(generator) {
                Generation::Accepted { value, .. } => value,
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: map,
                    };
                }
            };

            map.insert(key, value);
        }

        generator.accept(map)
    }
}

pub fn btree_map<K, V, KS, VS, H>(
    mut key_strategy: KS,
    mut value_strategy: VS,
    hint: H,
) -> impl FnMut(&mut DefaultGenerator) -> Generation<BTreeMap<K, V>>
where
    K: Ord,
    KS: FnMut(&mut DefaultGenerator) -> Generation<K>,
    VS: FnMut(&mut DefaultGenerator) -> Generation<V>,
    H: SizeHint,
{
    move |generator: &mut DefaultGenerator| {
        let target_len = hint.pick(&mut generator.rng);
        let mut map = BTreeMap::new();

        if target_len == 0 {
            return generator.accept(map);
        }

        let mut attempts_remaining = MAX_STRATEGY_ATTEMPTS * target_len.max(1);

        while map.len() < target_len {
            if attempts_remaining == 0 {
                break;
            }
            attempts_remaining -= 1;

            let key = match key_strategy(generator) {
                Generation::Accepted { value, .. } => value,
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: map,
                    };
                }
            };

            let value = match value_strategy(generator) {
                Generation::Accepted { value, .. } => value,
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: map,
                    };
                }
            };

            map.insert(key, value);
        }

        generator.accept(map)
    }
}
