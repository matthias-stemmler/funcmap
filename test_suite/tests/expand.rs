#[cfg_attr(not(expandtest), ignore)]
#[rustversion::attr(not(nightly), ignore)]
#[test]
fn expand() {
    macrotest::expand("tests/expand/**/*.rs");
}
