use std::collections::{HashMap, HashSet, VecDeque};

use estoa_proptest::{
    proptest,
    strategy::{
        runtime::{Generation, Generator},
        *,
    },
};
use rand::Rng;

#[derive(Default)]
struct DifferentStrategy;

impl estoa_proptest::strategy::Strategy for DifferentStrategy {
    type Value = (u8, u8);
    type Tree = StaticTree<(u8, u8)>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        loop {
            let first = generator.rng.random::<u8>();
            let second = generator.rng.random::<u8>();

            if first != second {
                return generator.accept(StaticTree::new((first, second)));
            }
        }
    }
}

#[proptest]
fn test_different(#[strategy(DifferentStrategy)] (a, b): (u8, u8)) {
    assert_ne!(a, b);
}

#[proptest]
fn test_vec_range(
    #[strategy(VecStrategy::new(AnyU8::default(), 3usize..=5usize))] list: Vec<
        u8,
    >,
) {
    assert!((3..=5).contains(&list.len()));
}

#[proptest]
fn test_vec_exact_size(
    #[strategy(VecStrategy::new(AnyU8::default(), 4usize..=4usize))] list: Vec<
        u8,
    >,
) {
    assert_eq!(list.len(), 4);
}

#[proptest]
fn test_vec_deque(
    #[strategy(VecDequeStrategy::new(AnyU8::default(), 4usize..=4usize))]
    deque: VecDeque<u8>,
) {
    assert_eq!(deque.len(), 4);
}

#[proptest]
fn test_hash_set(
    #[strategy(HashSetStrategy::new(AnyU16::default(), 8usize..=8usize))]
    set: HashSet<u16>,
) {
    assert_eq!(set.len(), 8);
    assert_eq!(set.len(), set.iter().copied().collect::<HashSet<_>>().len());
}

#[proptest]
fn test_hash_map(
    #[strategy(HashMapStrategy::new(
        AnyI32::default(),
        AnyU8::default(),
        6usize..=6usize,
    ))]
    map: HashMap<i32, u8>,
) {
    assert_eq!(map.len(), 6);
    let keys: HashSet<_> = map.keys().copied().collect();
    assert_eq!(keys.len(), 6);
}
