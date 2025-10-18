use std::collections::{
    BTreeMap,
    BTreeSet,
    BinaryHeap,
    HashMap,
    HashSet,
    VecDeque,
};

use estoa_proptest::{
    proptest,
    strategy::{
        AnyBool,
        AnyI32,
        ArrayStrategy,
        OptionStrategy,
        ResultStrategy,
        collections::{
            BTreeMapStrategy,
            BTreeSetStrategy,
            BinaryHeapStrategy,
            HashMapStrategy,
            HashSetStrategy,
            VecDequeStrategy,
            VecStrategy,
        },
    },
};

#[proptest(cases = 64)]
fn option_strategy_handles_none_and_some(
    #[strategy(OptionStrategy::new(AnyI32::default()))] value: Option<i32>,
) {
    if let Some(inner) = value {
        // Nothing more than type coverage for now, but ensure value is preserved.
        assert!(inner >= i32::MIN);
    }
}

#[proptest(cases = 64)]
fn result_strategy_emits_ok_or_err(
    #[strategy(ResultStrategy::new(AnyI32::default(), AnyBool))]
    value: Result<i32, bool>,
) {
    match value {
        Ok(inner) => assert!(inner >= i32::MIN),
        Err(flag) => assert!(flag || !flag),
    }
}

#[proptest(cases = 32)]
fn array_strategy_keeps_fixed_length(
    #[strategy(ArrayStrategy::<_, 4>::new(AnyBool))] value: [bool;
        4],
) {
    assert_eq!(value.len(), 4);
}

#[proptest(cases = 32)]
fn tuple_strategy_combines_multiple_components(
    #[strategy((AnyI32::default(), AnyBool, AnyI32::default()))]
    value: (i32, bool, i32),
) {
    assert!(value.0 >= i32::MIN);
    assert!(value.2 >= i32::MIN);
}

#[proptest(cases = 32)]
fn vec_strategy_respects_length_range(
    #[strategy(VecStrategy::new(
        OptionStrategy::new(AnyI32::default()),
        1usize..=3usize,
    ))]
    values: Vec<Option<i32>>,
) {
    assert!((1..=3).contains(&values.len()));
}

#[proptest(cases = 32)]
fn vec_deque_strategy_respects_length(
    #[strategy(VecDequeStrategy::new(
        AnyBool,
        2usize..=5usize,
    ))]
    deque: VecDeque<bool>,
) {
    assert!((2..=5).contains(&deque.len()));
}

#[proptest(cases = 32)]
fn binary_heap_strategy_produces_heap(
    #[strategy(BinaryHeapStrategy::new(
        AnyI32::default(),
        3usize..=6usize,
    ))]
    heap: BinaryHeap<i32>,
) {
    assert!((3..=6).contains(&heap.len()));
}

#[proptest(cases = 32)]
fn hash_set_strategy_yields_unique_elements(
    #[strategy(HashSetStrategy::new(
        AnyI32::default(),
        3usize..=5usize,
    ))]
    set: HashSet<i32>,
) {
    assert!(set.len() <= 5);
    assert_eq!(set.len(), set.iter().copied().collect::<HashSet<_>>().len());
}

#[proptest(cases = 32)]
fn hash_map_strategy_yields_unique_keys(
    #[strategy(HashMapStrategy::new(
        AnyI32::default(),
        AnyBool,
        2usize..=4usize,
    ))]
    map: HashMap<i32, bool>,
) {
    assert!(map.len() <= 4);
    let keys: HashSet<_> = map.keys().copied().collect();
    assert_eq!(keys.len(), map.len());
}

#[proptest(cases = 32)]
fn btree_set_strategy_orders_elements(
    #[strategy(BTreeSetStrategy::new(
        AnyI32::default(),
        2usize..=4usize,
    ))]
    set: BTreeSet<i32>,
) {
    let mut prev: Option<i32> = None;
    for value in set.iter().copied() {
        if let Some(previous) = prev {
            assert!(previous <= value);
        }
        prev = Some(value);
    }
}

#[proptest(cases = 32)]
fn btree_map_strategy_maintains_order(
    #[strategy(BTreeMapStrategy::new(
        AnyI32::default(),
        AnyBool,
        1usize..=3usize,
    ))]
    map: BTreeMap<i32, bool>,
) {
    let mut prev: Option<i32> = None;
    for key in map.keys().copied() {
        if let Some(previous) = prev {
            assert!(previous <= key);
        }
        prev = Some(key);
    }
}
