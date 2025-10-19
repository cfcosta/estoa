use estoa_proptest_macros::proptest;

#[proptest(recursion_limit = 0)]
fn recursion_limit_zero() {}

fn main() {}
