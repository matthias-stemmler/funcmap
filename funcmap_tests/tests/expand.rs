// Run expand tests only if preconditions for macrotest are satisfied:
// - a nightly toolchain is used
// - cargo-expand is installed
#[rustversion::attr(not(nightly), ignore)]
#[cfg_attr(not(has_cargo_expand), ignore)]
#[allow(unused_attributes)] // don't warn on multiple #[ignore]
#[test]
fn expand() {
    macrotest::expand("tests/expand/**/*.rs");
}
