#[test]
fn ui() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/cases_zero.rs");
    tests.compile_fail("tests/ui/duplicate_keys.rs");
    tests.compile_fail("tests/ui/unknown_key.rs");
    tests.compile_fail("tests/ui/recursion_limit_zero.rs");
    tests.compile_fail("tests/ui/rejection_limit_zero.rs");
}
