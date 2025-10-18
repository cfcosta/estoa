#![allow(
    clippy::type_complexity,
    unused_macro_rules,
    clippy::absurd_extreme_comparisons,
    clippy::too_many_arguments
)]

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
fn test_custom_arbitrary_implementation() {
    let _ = User::random().take();
}

#[test]
fn test_generate_primitive_types() {
    let _: () = random().take();
    let _: bool = random().take();
    let _: char = random().take();
    let _: u8 = random().take();
    let _: u16 = random().take();
    let _: u32 = random().take();
    let _: u64 = random().take();
    let _: u128 = random().take();
    let _: usize = random().take();
    let _: i8 = random().take();
    let _: i16 = random().take();
    let _: i32 = random().take();
    let _: i64 = random().take();
    let _: i128 = random().take();
    let _: isize = random().take();
    let _: f32 = random().take();
    let _: f64 = random().take();
    let _: String = random().take();
}

#[test]
fn test_generate_generic_types() {
    let _: Option<u8> = random().take();
    let _: Result<String, u32> = random().take();
    let _: Box<u16> = random().take();
    let _: Rc<i32> = random().take();
    let _: Arc<u64> = random().take();
    let _: Vec<u8> = random().take();
    let _: VecDeque<u16> = random().take();
    let _: BinaryHeap<i32> = random().take();
    let _: HashSet<u32> = random().take();
    let _: BTreeSet<i64> = random().take();
    let _: HashMap<u8, u16> = random().take();
    let _: BTreeMap<u32, i32> = random().take();
    let _: [u8; 8] = random().take();
    let _: (u8, String) = random().take();
    let _: (u8, u16, u32, u64) = random().take();
    let _: (
        u8,
        u16,
        u32,
        u64,
        String,
        BTreeMap<u8, String>,
        HashMap<String, HashMap<String, u64>>,
    ) = random().take();
}

#[proptest(cases = 1)]
fn test_arbitrary_generation_with_macro(
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
