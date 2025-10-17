use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque},
    hint::black_box,
    rc::Rc,
    sync::Arc,
};

use estoa_proptest::{Arbitrary, proptest, random};
use rand::{CryptoRng, RngCore};

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

#[proptest(cases = 1)]
fn test_primitives_arbitrary(
    unit: (),
    boolean: bool,
    character: char,
    u8_value: u8,
    u16_value: u16,
    u32_value: u32,
    u64_value: u64,
    u128_value: u128,
    usize_value: usize,
    i8_value: i8,
    i16_value: i16,
    i32_value: i32,
    i64_value: i64,
    i128_value: i128,
    isize_value: isize,
    f32_value: f32,
    f64_value: f64,
    string_value: String,
) {
    black_box(&unit);
    black_box(&boolean);
    black_box(&character);
    black_box(&u8_value);
    black_box(&u16_value);
    black_box(&u32_value);
    black_box(&u64_value);
    black_box(&u128_value);
    black_box(&usize_value);
    black_box(&i8_value);
    black_box(&i16_value);
    black_box(&i32_value);
    black_box(&i64_value);
    black_box(&i128_value);
    black_box(&isize_value);
    black_box(&f32_value);
    black_box(&f64_value);
    assert!(string_value.capacity() >= string_value.len());
}
