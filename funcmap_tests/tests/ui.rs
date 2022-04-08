use trybuild::TestCases;

#[rustversion::attr(nightly, ignore)] // UI tests are too unstable on nightly
#[test]
fn ui() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}
