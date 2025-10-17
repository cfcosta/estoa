use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    hash::Hash,
};

use paste::paste;
use rand::{CryptoRng, Rng, RngCore};

use super::{Generation, Generator, MAX_STRATEGY_ATTEMPTS};
use crate::{Arbitrary, arbitrary, arbitrary::COLLECTION_MAX_LEN};

macro_rules! define_unary_collection_strategies {
    ($(
        $module:ident => {
            collection: $collection:ident;
            element: $element:ident;
            constraints: [$($constraints:tt)*];
            method: $method:ident;
        }
    )+) => {
        $(
            paste! {
                pub mod $module {
                    use super::*;

                    pub fn not_empty<$element, R>(
                        generator: &mut Generator<R>,
                    ) -> Generation<$collection<$element>>
                    where
                        R: RngCore + CryptoRng,
                        $($constraints)*
                    {
                        let len = generator.rng.random_range(1..=COLLECTION_MAX_LEN);
                        let mut collection: $collection<$element> = Default::default();
                        let mut attempts = 0usize;

                        while collection.len() < len
                            && attempts < MAX_STRATEGY_ATTEMPTS
                        {
                            attempts += 1;
                            let value = arbitrary(&mut generator.rng);
                            collection.$method(value);
                        }

                        if collection.is_empty() {
                            for _ in 0..MAX_STRATEGY_ATTEMPTS {
                                let value = arbitrary(&mut generator.rng);
                                collection.$method(value);

                                if !collection.is_empty() {
                                    break;
                                }
                            }
                        }

                        generator.accept(collection)
                    }
                }
            }
        )+
    };
}

macro_rules! define_map_collection_strategies {
    ($(
        $module:ident => {
            collection: $collection:ident;
            key: $key:ident;
            value: $value:ident;
            key_constraints: [$($key_constraints:tt)*];
            value_constraints: [$($value_constraints:tt)*];
            method: $method:ident;
        }
    )+) => {
        $(
            paste! {
                pub mod $module {
                    use super::*;

                    pub fn not_empty<$key, $value, R>(
                        generator: &mut Generator<R>,
                    ) -> Generation<$collection<$key, $value>>
                    where
                        R: RngCore + CryptoRng,
                        $($key_constraints)*
                        $($value_constraints)*
                    {
                        let len = generator.rng.random_range(1..=COLLECTION_MAX_LEN);
                        let mut collection: $collection<$key, $value> =
                            Default::default();
                        let mut attempts = 0usize;

                        while collection.len() < len
                            && attempts < MAX_STRATEGY_ATTEMPTS
                        {
                            attempts += 1;
                            let key = arbitrary(&mut generator.rng);
                            let value = arbitrary(&mut generator.rng);
                            collection.$method(key, value);
                        }

                        if collection.is_empty() {
                            for _ in 0..MAX_STRATEGY_ATTEMPTS {
                                let key = arbitrary(&mut generator.rng);
                                let value = arbitrary(&mut generator.rng);
                                collection.$method(key, value);

                                if !collection.is_empty() {
                                    break;
                                }
                            }
                        }

                        generator.accept(collection)
                    }
                }
            }
        )+
    };
}

define_unary_collection_strategies! {
    vec => {
        collection: Vec;
        element: T;
        constraints: [T: Arbitrary + PartialEq];
        method: push;
    }
    vec_deque => {
        collection: VecDeque;
        element: T;
        constraints: [T: Arbitrary];
        method: push_back;
    }
    binary_heap => {
        collection: BinaryHeap;
        element: T;
        constraints: [T: Arbitrary + Ord];
        method: push;
    }
    hash_set => {
        collection: HashSet;
        element: T;
        constraints: [T: Arbitrary + Eq + Hash];
        method: insert;
    }
    b_tree_set => {
        collection: BTreeSet;
        element: T;
        constraints: [T: Arbitrary + Ord];
        method: insert;
    }
}

define_map_collection_strategies! {
    hash_map => {
        collection: HashMap;
        key: K;
        value: V;
        key_constraints: [K: Arbitrary + Eq + Hash,];
        value_constraints: [V: Arbitrary,];
        method: insert;
    }
    b_tree_map => {
        collection: BTreeMap;
        key: K;
        value: V;
        key_constraints: [K: Arbitrary + Ord,];
        value_constraints: [V: Arbitrary,];
        method: insert;
    }
}
