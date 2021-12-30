#[cfg_attr(not(expandtest), ignore)]
#[test]
fn expand() {
    macrotest::expand("tests/expand/**/*.rs");
}
