use std::{
    array,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    hash::Hash,
    rc::Rc,
    sync::Arc,
};

use rand::{
    CryptoRng,
    Rng,
    RngCore,
    distr::{SampleString, StandardUniform},
};

use crate::strategy::runtime::{Generation, Generator};

pub(crate) const STRING_MAX_LEN: usize = 128;
pub(crate) const COLLECTION_MAX_LEN: usize = 32;

pub trait Arbitrary
where
    Self: Sized,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self;

    fn generate<R: RngCore + CryptoRng>(
        generator: &mut Generator<R>,
    ) -> Generation<Self> {
        let value = Self::arbitrary(&mut generator.rng);
        generator.accept(value)
    }

    fn random() -> Generation<Self> {
        let mut generator =
            Generator::build_with_limit(rand::rng(), usize::MAX);
        Self::generate(&mut generator)
    }
}

macro_rules! delegate_arbitrary {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl Arbitrary for $ty {
                fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
                    rng.random::<$ty>()
                }
            }
        )+
    };
}

delegate_arbitrary!(bool);
delegate_arbitrary!(char);
delegate_arbitrary!(u8, u16, u32, u64, u128);
delegate_arbitrary!(i8, i16, i32, i64, i128);
delegate_arbitrary!(f32, f64);

impl Arbitrary for () {
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(_: &mut R) -> Self {}
}

impl Arbitrary for String {
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=STRING_MAX_LEN);
        StandardUniform.sample_string(rng, len)
    }
}

impl Arbitrary for usize {
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let mut bytes = [0u8; core::mem::size_of::<usize>()];
        rng.fill_bytes(&mut bytes);
        usize::from_ne_bytes(bytes)
    }
}

impl Arbitrary for isize {
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let mut bytes = [0u8; core::mem::size_of::<isize>()];
        rng.fill_bytes(&mut bytes);
        isize::from_ne_bytes(bytes)
    }
}

impl<T> Arbitrary for Option<T>
where
    T: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        if rng.random::<bool>() {
            Some(T::arbitrary(rng))
        } else {
            None
        }
    }
}

impl<T, E> Arbitrary for Result<T, E>
where
    T: Arbitrary,
    E: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        if bool::arbitrary(rng) {
            Ok(T::arbitrary(rng))
        } else {
            Err(E::arbitrary(rng))
        }
    }
}

impl<T> Arbitrary for Box<T>
where
    T: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        Box::new(T::arbitrary(rng))
    }
}

impl<T> Arbitrary for Rc<T>
where
    T: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        Rc::new(T::arbitrary(rng))
    }
}

impl<T> Arbitrary for Arc<T>
where
    T: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        Arc::new(T::arbitrary(rng))
    }
}

impl<T> Arbitrary for Vec<T>
where
    T: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=COLLECTION_MAX_LEN);
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(T::arbitrary(rng));
        }
        values
    }
}

impl<T> Arbitrary for VecDeque<T>
where
    T: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=COLLECTION_MAX_LEN);
        let mut values = VecDeque::with_capacity(len);
        for _ in 0..len {
            values.push_back(T::arbitrary(rng));
        }
        values
    }
}

impl<T> Arbitrary for BinaryHeap<T>
where
    T: Arbitrary + Ord,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=COLLECTION_MAX_LEN);
        let mut heap = BinaryHeap::with_capacity(len);
        for _ in 0..len {
            heap.push(T::arbitrary(rng));
        }
        heap
    }
}

impl<T> Arbitrary for HashSet<T>
where
    T: Arbitrary + Eq + Hash,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=COLLECTION_MAX_LEN);
        let mut set = HashSet::with_capacity(len);
        for _ in 0..len {
            set.insert(T::arbitrary(rng));
        }
        set
    }
}

impl<K, V> Arbitrary for HashMap<K, V>
where
    K: Arbitrary + Eq + Hash,
    V: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=COLLECTION_MAX_LEN);
        let mut map = HashMap::with_capacity(len);

        for _ in 0..len {
            map.insert(K::arbitrary(rng), V::arbitrary(rng));
        }

        map
    }
}

impl<T> Arbitrary for BTreeSet<T>
where
    T: Arbitrary + Ord,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=COLLECTION_MAX_LEN);
        let mut set = BTreeSet::new();

        for _ in 0..len {
            set.insert(T::arbitrary(rng));
        }

        set
    }
}

impl<K, V> Arbitrary for BTreeMap<K, V>
where
    K: Arbitrary + Ord,
    V: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.random_range(0..=COLLECTION_MAX_LEN);
        let mut map = BTreeMap::new();

        for _ in 0..len {
            map.insert(K::arbitrary(rng), V::arbitrary(rng));
        }

        map
    }
}

impl<T, const N: usize> Arbitrary for [T; N]
where
    T: Arbitrary,
{
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        array::from_fn(|_| T::arbitrary(rng))
    }
}

macro_rules! impl_arbitrary_tuple {
    ($first:ident, $($rest:ident),+) => {
        impl<$first, $($rest),+> Arbitrary for ($first, $($rest,)+)
        where
            $first: Arbitrary,
            $( $rest: Arbitrary ),+
        {
            fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
                (
                    $first::arbitrary(rng),
                    $( $rest::arbitrary(rng), )+
                )
            }
        }
    };
}

impl_arbitrary_tuple!(A, B);
impl_arbitrary_tuple!(A, B, C);
impl_arbitrary_tuple!(A, B, C, D);
impl_arbitrary_tuple!(A, B, C, D, E);
impl_arbitrary_tuple!(A, B, C, D, E, F);
impl_arbitrary_tuple!(A, B, C, D, E, F, G);
impl_arbitrary_tuple!(A, B, C, D, E, F, G, H);
impl_arbitrary_tuple!(A, B, C, D, E, F, G, H, I);
impl_arbitrary_tuple!(A, B, C, D, E, F, G, H, I, J);
