use proc_macro2::Ident;
use syn::fold::Fold;
use syn::{fold, parse_quote, Type, TypeParam};

use crate::path::is_ident;

/// Substitutes `type_param` with `type_param_subs` within `ty`
pub fn subs_type_param(ty: &Type, type_param: &TypeParam, type_param_subs: &TypeParam) -> Type {
    let mut folder = SubsTypeParamFolder {
        ident: &type_param.ident,
        type_param_subs,
    };

    folder.fold_type(ty.clone())
}

struct SubsTypeParamFolder<'a> {
    ident: &'a Ident,
    type_param_subs: &'a TypeParam,
}

impl Fold for SubsTypeParamFolder<'_> {
    fn fold_type(&mut self, ty: Type) -> Type {
        match &ty {
            Type::Path(type_path) if is_ident(&type_path.path, self.ident) => {
                let type_param_subs = self.type_param_subs;
                parse_quote!(#type_param_subs)
            }
            _ => fold::fold_type(self, ty),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use syn::parse_quote;

    use super::*;

    #[rstest]
    #[case(parse_quote ! (Foo), parse_quote ! (Foo))]
    #[case(parse_quote ! (T), parse_quote ! (A))]
    #[case(parse_quote ! ((T)), parse_quote ! ((A)))]
    #[case(parse_quote ! ((Foo, T)), parse_quote ! ((Foo, A)))]
    #[case(parse_quote ! ([T]), parse_quote ! ([A]))]
    #[case(parse_quote ! ([T; 1]), parse_quote ! ([A; 1]))]
    #[case(parse_quote ! (fn (T) -> Foo), parse_quote ! (fn (A) -> Foo))]
    #[case(parse_quote ! (fn (Foo) -> T), parse_quote ! (fn (Foo) -> A))]
    #[case(parse_quote ! (* const T), parse_quote ! (* const A))]
    #[case(parse_quote ! (* mut T), parse_quote ! (* mut A))]
    #[case(parse_quote ! (& T), parse_quote ! (& A))]
    #[case(parse_quote ! (& mut T), parse_quote ! (& mut A))]
    #[case(parse_quote ! (Foo::T), parse_quote ! (Foo::T))]
    #[case(parse_quote ! (Foo < Bar >), parse_quote ! (Foo < Bar >))]
    #[case(parse_quote ! (Foo < T >), parse_quote ! (Foo < A >))]
    #[case(parse_quote ! (Foo < Bar < T >>), parse_quote ! (Foo < Bar < A >>))]
    #[case(parse_quote ! (T < Foo >), parse_quote ! (T < Foo >))]
    #[case(parse_quote ! (T::Foo < Bar >), parse_quote ! (T::Foo < Bar >))]
    #[case(parse_quote ! (< T as Foo >::Bar < Baz >), parse_quote ! (< A as Foo >::Bar < Baz >))]
    #[case(parse_quote ! (Foo < Bar, T >), parse_quote ! (Foo < Bar, A >))]
    #[case(parse_quote ! (Foo < 'T >), parse_quote ! (Foo < 'T >))]
    fn test_subs_type_param(#[case] ty: Type, #[case] expected_result: Type) {
        let type_param: TypeParam = parse_quote!(T);
        let type_param_subs: TypeParam = parse_quote!(A);

        assert_eq!(
            subs_type_param(&ty, &type_param, &type_param_subs),
            expected_result
        );
    }
}
