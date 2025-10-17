use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    rc::Rc,
    sync::Arc,
};

use estoa_proptest::Arbitrary;
use rand::{CryptoRng, RngCore, rngs::ThreadRng};

#[allow(unused)]
struct User {
    name: String,
    followers: u32,
    following: u64,
}

impl Arbitrary for User {
    fn arbitrary<R: RngCore + CryptoRng + ?Sized>(rng: &mut R) -> Self {
        Self {
            name: String::arbitrary(rng),
            followers: u32::arbitrary(rng),
            following: u64::arbitrary(rng),
        }
    }
}

#[test]
fn test_user_created_random() {
    let _ = User::random();
}

#[test]
fn test_primitives_random() {
    let _: () = <()>::random();
    let _: bool = bool::random();
    let _: char = char::random();
    let _: u8 = u8::random();
    let _: u16 = u16::random();
    let _: u32 = u32::random();
    let _: u64 = u64::random();
    let _: u128 = u128::random();
    let _: usize = usize::random();
    let _: i8 = i8::random();
    let _: i16 = i16::random();
    let _: i32 = i32::random();
    let _: i64 = i64::random();
    let _: i128 = i128::random();
    let _: isize = isize::random();
    let _: f32 = f32::random();
    let _: f64 = f64::random();
    let _: String = String::random();
}

#[test]
fn test_generic_random() {
    let _: Option<u8> = Option::random();
    let _: Result<String, u32> = Result::random();
    let _: Box<u16> = Box::random();
    let _: Rc<i32> = Rc::random();
    let _: Arc<u64> = Arc::random();
    let _: Vec<u8> = Vec::random();
    let _: VecDeque<u16> = VecDeque::random();
    let _: BinaryHeap<i32> = BinaryHeap::random();
    let _: HashSet<u32> = HashSet::random();
    let _: BTreeSet<i64> = BTreeSet::random();
    let _: HashMap<u8, u16> = HashMap::random();
    let _: BTreeMap<u32, i32> = BTreeMap::random();
    let _: [u8; 8] = <[u8; 8]>::random();
    let _: (u8, String) = <(u8, String)>::random();
    let _: (u8, u16, u32, u64) = <(u8, u16, u32, u64)>::random();
    let _: (
        u8,
        u16,
        u32,
        u64,
        String,
        BTreeMap<u8, String>,
        HashMap<String, HashMap<String, u64>>,
    ) = <(
        u8,
        u16,
        u32,
        u64,
        String,
        BTreeMap<u8, String>,
        HashMap<String, HashMap<String, u64>>,
    )>::random();
}

#[test]
fn test_primitives_arbitrary() {
    let mut rng = ThreadRng::default();

    let _: () = <()>::arbitrary(&mut rng);
    let _: bool = bool::arbitrary(&mut rng);
    let _: char = char::arbitrary(&mut rng);
    let _: u8 = u8::arbitrary(&mut rng);
    let _: u16 = u16::arbitrary(&mut rng);
    let _: u32 = u32::arbitrary(&mut rng);
    let _: u64 = u64::arbitrary(&mut rng);
    let _: u128 = u128::arbitrary(&mut rng);
    let _: usize = usize::arbitrary(&mut rng);
    let _: i8 = i8::arbitrary(&mut rng);
    let _: i16 = i16::arbitrary(&mut rng);
    let _: i32 = i32::arbitrary(&mut rng);
    let _: i64 = i64::arbitrary(&mut rng);
    let _: i128 = i128::arbitrary(&mut rng);
    let _: isize = isize::arbitrary(&mut rng);
    let _: f32 = f32::arbitrary(&mut rng);
    let _: f64 = f64::arbitrary(&mut rng);
    let _: String = String::arbitrary(&mut rng);
}
