use proc_macro2::Ident;
use syn::fold::{self, Fold};
use syn::visit::{self, Visit};
use syn::{
    ConstParam, GenericArgument, GenericParam, LifetimeDef, Path, PathArguments, PathSegment,
    TraitBound, Type, TypeParam, TypePath, WherePredicate,
};

pub trait DependencyOnType {
    fn dependency_on_type<'ast>(&'ast self, type_ident: &'ast Ident) -> Option<&'ast Type>;
}

impl DependencyOnType for Type {
    fn dependency_on_type<'ast>(&'ast self, type_ident: &'ast Ident) -> Option<&'ast Type> {
        let mut visitor = DependencyOnTypeVisitor::new(type_ident);
        visitor.visit_type(self);
        visitor.into_dependency()
    }
}

impl DependencyOnType for PathSegment {
    fn dependency_on_type<'ast>(&'ast self, type_ident: &'ast Ident) -> Option<&'ast Type> {
        let mut visitor = DependencyOnTypeVisitor::new(type_ident);
        visitor.visit_path_segment(self);
        visitor.into_dependency()
    }
}

#[derive(Debug)]
struct DependencyOnTypeVisitor<'ast> {
    type_ident: &'ast Ident,
    dependency: Option<&'ast Type>,
}

impl<'ast> DependencyOnTypeVisitor<'ast> {
    fn new(type_ident: &'ast Ident) -> Self {
        Self {
            type_ident,
            dependency: None,
        }
    }

    fn into_dependency(self) -> Option<&'ast Type> {
        self.dependency
    }
}

impl<'ast> Visit<'ast> for DependencyOnTypeVisitor<'ast> {
    fn visit_type(&mut self, ty: &'ast Type) {
        match self.dependency {
            None if ty.is_ident(self.type_ident) => self.dependency = Some(ty),
            None => visit::visit_type(self, ty),
            _ => (),
        };
    }
}

pub trait IntoGenericArgument {
    fn into_generic_argument(self) -> GenericArgument;
}

impl IntoGenericArgument for GenericParam {
    fn into_generic_argument(self) -> GenericArgument {
        match self {
            GenericParam::Type(TypeParam { ident, .. }) => GenericArgument::Type(ident.into_type()),
            GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
                GenericArgument::Lifetime(lifetime)
            }
            GenericParam::Const(ConstParam { ident, .. }) => {
                GenericArgument::Type(ident.into_type())
            }
        }
    }
}

pub trait IntoType {
    fn into_type(self) -> Type;
}

impl IntoType for Ident {
    fn into_type(self) -> Type {
        Type::Path(TypePath {
            qself: None,
            path: self.into(),
        })
    }
}

pub trait IsIdent {
    fn is_ident(&self, ident: &Ident) -> bool;
}

impl IsIdent for Type {
    fn is_ident(&self, ident: &Ident) -> bool {
        match self {
            Type::Path(TypePath {
                qself: None,
                path:
                    Path {
                        leading_colon: None,
                        segments,
                    },
            }) => {
                let mut segments = segments.iter();

                match segments.next() {
                    Some(PathSegment {
                        ident: segment_ident,
                        arguments: PathArguments::None,
                    }) => segment_ident == ident && segments.next().is_none(),
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

pub trait SubsType {
    fn subs_type(self, ident: &Ident, subs_ident: &Ident) -> Self;
}

impl SubsType for Type {
    fn subs_type(self, ident: &Ident, subs_ident: &Ident) -> Self {
        let mut folder = SubsTypeFolder { ident, subs_ident };
        folder.fold_type(self)
    }
}

impl SubsType for TraitBound {
    fn subs_type(self, ident: &Ident, subs_ident: &Ident) -> Self {
        let mut folder = SubsTypeFolder { ident, subs_ident };
        folder.fold_trait_bound(self)
    }
}

impl SubsType for WherePredicate {
    fn subs_type(self, ident: &Ident, subs_ident: &Ident) -> Self {
        let mut folder = SubsTypeFolder { ident, subs_ident };
        folder.fold_where_predicate(self)
    }
}

struct SubsTypeFolder<'ast> {
    ident: &'ast Ident,
    subs_ident: &'ast Ident,
}

impl Fold for SubsTypeFolder<'_> {
    fn fold_type(&mut self, ty: Type) -> Type {
        if ty.is_ident(self.ident) {
            self.subs_ident.clone().into_type()
        } else {
            fold::fold_type(self, ty)
        }
    }
}

pub trait WithIdent {
    fn with_ident(self, ident: Ident) -> Self;
}

impl WithIdent for TypeParam {
    fn with_ident(self, ident: Ident) -> Self {
        Self { ident, ..self }
    }
}

pub trait WithoutDefault {
    fn without_default(self) -> Self;
}

impl WithoutDefault for TypeParam {
    fn without_default(self) -> Self {
        Self {
            eq_token: None,
            default: None,
            ..self
        }
    }
}

impl WithoutDefault for ConstParam {
    fn without_default(self) -> Self {
        Self {
            eq_token: None,
            default: None,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
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
        let type_ident = Ident::new("T", Span::call_site());

        assert_eq!(
            ty.dependency_on_type(&type_ident).is_some(),
            expected_result
        );
    }
}
