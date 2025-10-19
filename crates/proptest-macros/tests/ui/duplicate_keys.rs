use estoa_proptest_macros::proptest;

#[proptest(cases = 2, cases = 3)]
fn duplicate_keys() {}

fn main() {}
