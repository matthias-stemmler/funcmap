#[cfg_attr(not(expandtest), ignore)]
#[rustversion::attr(not(nightly), ignore)]
#[allow(unused_attributes)]
#[test]
fn expand() {
    macrotest::expand("tests/expand/**/*.rs");
}
