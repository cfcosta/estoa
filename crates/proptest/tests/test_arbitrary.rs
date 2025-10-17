use estoa_proptest::Arbitrary;

#[allow(unused)]
struct User {
    name: String,
    followers: u32,
    following: u64,
}

impl Arbitrary for User {
    fn arbitrary<R: rand::RngCore + rand::CryptoRng>(rng: &mut R) -> Self {
        Self {
            name: String::arbitrary(rng),
            followers: u32::arbitrary(rng),
            following: u64::arbitrary(rng),
        }
    }
}

#[test]
fn test_unconstrained_random() {
    let _ = User::random();
}
