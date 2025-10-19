use estoa_proptest_macros::proptest;

#[proptest(rejection_limit = 0)]
fn rejection_limit_zero() {}

fn main() {}
