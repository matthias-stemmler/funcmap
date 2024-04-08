use trybuild::TestCases;

#[rustversion::attr(any(beta, nightly), ignore)]
#[test]
fn ui() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}
