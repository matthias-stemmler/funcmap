use proc_macro2::Ident;
use syn::visit::Visit;
use syn::{visit, Path, Type, TypeParam};

use crate::path::is_ident;

/// Determines if `ty` depends on `type_param`,
/// i.e. if `ty` would change when `type_param` were substituted with a different type parameter
pub fn depends_on(ty: &Type, type_param: &TypeParam) -> bool {
    let mut visitor = DependsOnVisitor {
        ident: &type_param.ident,
        found: false,
    };

    visitor.visit_type(ty);

    visitor.found
}

struct DependsOnVisitor<'a> {
    ident: &'a Ident,
    found: bool,
}

impl Visit<'_> for DependsOnVisitor<'_> {
    fn visit_path(&mut self, path: &Path) {
        if is_ident(path, self.ident) {
            self.found = true;
        } else {
            visit::visit_path(self, path);
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use syn::parse_quote;

    use super::*;

    #[rstest]
    #[case(parse_quote ! (Foo), false)]
    #[case(parse_quote ! (T), true)]
    #[case(parse_quote ! ((T)), true)]
    #[case(parse_quote ! ((Foo, T)), true)]
    #[case(parse_quote ! ([T]), true)]
    #[case(parse_quote ! ([T; 1]), true)]
    #[case(parse_quote ! (fn (T) -> Foo), true)]
    #[case(parse_quote ! (fn (Foo) -> T), true)]
    #[case(parse_quote ! (* const T), true)]
    #[case(parse_quote ! (* mut T), true)]
    #[case(parse_quote ! (& T), true)]
    #[case(parse_quote ! (& mut T), true)]
    #[case(parse_quote ! (Foo::T), false)]
    #[case(parse_quote ! (Foo < Bar >), false)]
    #[case(parse_quote ! (Foo < T >), true)]
    #[case(parse_quote ! (Foo < Bar < T >>), true)]
    #[case(parse_quote ! (T < Foo >), false)]
    #[case(parse_quote ! (T::Foo < Bar >), false)]
    #[case(parse_quote ! (< T as Foo >::Bar < Baz >), true)]
    #[case(parse_quote ! (Foo < Bar, T >), true)]
    #[case(parse_quote ! (Foo < 'T >), false)]
    fn test_depends_on(#[case] ty: Type, #[case] expected_result: bool) {
        let type_param: TypeParam = parse_quote!(T);

        assert_eq!(depends_on(&ty, &type_param), expected_result);
    }
}
