use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    rc::Rc,
    sync::Arc,
};

use estoa_proptest::{Arbitrary, arbitrary, random};
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
    let _: () = random();
    let _: bool = random();
    let _: char = random();
    let _: u8 = random();
    let _: u16 = random();
    let _: u32 = random();
    let _: u64 = random();
    let _: u128 = random();
    let _: usize = random();
    let _: i8 = random();
    let _: i16 = random();
    let _: i32 = random();
    let _: i64 = random();
    let _: i128 = random();
    let _: isize = random();
    let _: f32 = random();
    let _: f64 = random();
    let _: String = random();
}

#[test]
fn test_generic_random() {
    let _: Option<u8> = random();
    let _: Result<String, u32> = random();
    let _: Box<u16> = random();
    let _: Rc<i32> = random();
    let _: Arc<u64> = random();
    let _: Vec<u8> = random();
    let _: VecDeque<u16> = random();
    let _: BinaryHeap<i32> = random();
    let _: HashSet<u32> = random();
    let _: BTreeSet<i64> = random();
    let _: HashMap<u8, u16> = random();
    let _: BTreeMap<u32, i32> = random();
    let _: [u8; 8] = random();
    let _: (u8, String) = random();
    let _: (u8, u16, u32, u64) = random();
    let _: (
        u8,
        u16,
        u32,
        u64,
        String,
        BTreeMap<u8, String>,
        HashMap<String, HashMap<String, u64>>,
    ) = random();
}

#[test]
fn test_primitives_arbitrary() {
    let mut rng = ThreadRng::default();

    let _: () = arbitrary(&mut rng);
    let _: bool = arbitrary(&mut rng);
    let _: char = arbitrary(&mut rng);
    let _: u8 = arbitrary(&mut rng);
    let _: u16 = arbitrary(&mut rng);
    let _: u32 = arbitrary(&mut rng);
    let _: u64 = arbitrary(&mut rng);
    let _: u128 = arbitrary(&mut rng);
    let _: usize = arbitrary(&mut rng);
    let _: i8 = arbitrary(&mut rng);
    let _: i16 = arbitrary(&mut rng);
    let _: i32 = arbitrary(&mut rng);
    let _: i64 = arbitrary(&mut rng);
    let _: i128 = arbitrary(&mut rng);
    let _: isize = arbitrary(&mut rng);
    let _: f32 = arbitrary(&mut rng);
    let _: f64 = arbitrary(&mut rng);
    let _: String = arbitrary(&mut rng);
}
