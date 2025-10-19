use estoa_proptest_macros::proptest;

#[proptest(not_a_key = 1)]
fn unknown_key() {}

fn main() {}
