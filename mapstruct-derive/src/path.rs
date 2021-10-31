use proc_macro2::Ident;
use syn::Path;

use crate::iter;

/// Determines if `path` consists solely of the identifier `ident`
pub fn is_ident(path: &Path, ident: &Ident) -> bool {
    if path.leading_colon.is_some() {
        return false;
    }

    match iter::single(&path.segments) {
        Some(segment) => &segment.ident == ident && segment.arguments.is_empty(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use syn::parse_quote;

    use super::*;

    #[rstest]
    #[case(parse_quote ! (T), true)]
    #[case(parse_quote ! (::T), false)]
    #[case(parse_quote ! (Foo), false)]
    #[case(parse_quote ! (T < Foo >), false)]
    #[case(parse_quote ! (T::Foo), false)]
    #[case(parse_quote ! (T::T), false)]
    fn test_is_ident(#[case] path: Path, #[case] expected_result: bool) {
        let ident: Ident = parse_quote!(T);

        assert_eq!(is_ident(&path, &ident), expected_result);
    }
}
