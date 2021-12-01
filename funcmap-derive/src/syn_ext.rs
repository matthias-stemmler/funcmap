use proc_macro2::{Ident, Span};
use syn::fold::{self, Fold};
use syn::punctuated::Punctuated;
use syn::token::Add;
use syn::visit::{self, Visit};
use syn::{
    AngleBracketedGenericArguments, ConstParam, GenericArgument, GenericParam, LifetimeDef, Path,
    PathArguments, PathSegment, PredicateType, QSelf, TraitBound, TraitBoundModifier, Type,
    TypeParam, TypeParamBound, TypePath, WherePredicate,
};

pub trait DependencyOnType {
    fn dependency_on_type<'ast>(&'ast self, type_ident: &'ast Ident) -> Option<&'ast Ident>;
}

impl DependencyOnType for Type {
    fn dependency_on_type<'ast>(&'ast self, type_ident: &'ast Ident) -> Option<&'ast Ident> {
        let mut visitor = DependencyOnTypeVisitor::new(type_ident);
        visitor.visit_type(self);
        visitor.into_dependency()
    }
}

#[derive(Debug)]
struct DependencyOnTypeVisitor<'ast> {
    type_ident: &'ast Ident,
    dependency: Option<&'ast Ident>,
}

impl<'ast> DependencyOnTypeVisitor<'ast> {
    fn new(type_ident: &'ast Ident) -> Self {
        Self {
            type_ident,
            dependency: None,
        }
    }

    fn into_dependency(self) -> Option<&'ast Ident> {
        self.dependency
    }
}

impl<'ast> Visit<'ast> for DependencyOnTypeVisitor<'ast> {
    fn visit_type(&mut self, ty: &'ast Type) {
        match (self.dependency, ty.find_ident(self.type_ident)) {
            (None, Some(ident)) => self.dependency = Some(ident),
            (None, _) => visit::visit_type(self, ty),
            _ => (),
        };
    }

    fn visit_path(&mut self, path: &'ast Path) {
        match (self.dependency, path.leading_colon, path.segments.first()) {
            (None, None, Some(PathSegment { ident, .. })) if ident == self.type_ident => {
                self.dependency = Some(ident)
            }
            (None, ..) => visit::visit_path(self, path),
            _ => (),
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
        if ty.find_ident(self.ident).is_some() {
            self.subs_ident.clone().into_type()
        } else {
            fold::fold_type(self, ty)
        }
    }

    fn fold_path(&mut self, mut path: Path) -> Path {
        match (path.leading_colon, path.segments.first_mut()) {
            (None, Some(PathSegment { ident, .. })) if ident == self.ident => {
                *ident = self.subs_ident.clone();
            }
            _ => (),
        }

        fold::fold_path(self, path)
    }
}

trait FindIdent {
    fn find_ident(&self, ident: &Ident) -> Option<&Ident>;
}

impl FindIdent for Type {
    fn find_ident(&self, ident: &Ident) -> Option<&Ident> {
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
                    }) if segment_ident == ident && segments.next().is_none() => {
                        Some(segment_ident)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
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

pub trait WithoutAttrs {
    fn without_attrs(self) -> Self;
}

impl WithoutAttrs for ConstParam {
    fn without_attrs(self) -> Self {
        Self {
            attrs: Vec::new(),
            ..self
        }
    }
}

impl WithoutAttrs for LifetimeDef {
    fn without_attrs(self) -> Self {
        Self {
            attrs: Vec::new(),
            ..self
        }
    }
}

pub trait WithoutDefault {
    fn without_default(self) -> Self;
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

pub trait WithoutMaybeBounds {
    fn without_maybe_bounds(self) -> Self;
}

impl WithoutMaybeBounds for Punctuated<TypeParamBound, Add> {
    fn without_maybe_bounds(self) -> Self {
        self.into_iter()
            .filter(|bound| {
                matches!(
                    bound,
                    TypeParamBound::Trait(TraitBound {
                        modifier: TraitBoundModifier::None,
                        ..
                    })
                )
            })
            .collect()
    }
}

impl WithoutMaybeBounds for WherePredicate {
    fn without_maybe_bounds(self) -> Self {
        match self {
            WherePredicate::Type(predicate_type) => WherePredicate::Type(PredicateType {
                bounds: predicate_type.bounds.without_maybe_bounds(),
                ..predicate_type
            }),
            predicate => predicate,
        }
    }
}

pub trait WithSpan {
    fn with_span(self, span: Span) -> Self;
}

impl WithSpan for AngleBracketedGenericArguments {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_angle_bracketed_generic_arguments(self)
    }
}

impl WithSpan for ConstParam {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_const_param(self)
    }
}

impl WithSpan for GenericArgument {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_generic_argument(self)
    }
}

impl WithSpan for GenericParam {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_generic_param(self)
    }
}

impl WithSpan for Ident {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_ident(self)
    }
}

impl WithSpan for LifetimeDef {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_lifetime_def(self)
    }
}

impl WithSpan for QSelf {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_qself(self)
    }
}

impl WithSpan for TraitBound {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_trait_bound(self)
    }
}

impl WithSpan for Type {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_type(self)
    }
}

impl WithSpan for TypeParamBound {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_type_param_bound(self)
    }
}

impl WithSpan for WherePredicate {
    fn with_span(self, span: Span) -> Self {
        let mut folder = WithSpanFolder::new(span);
        folder.fold_where_predicate(self)
    }
}

struct WithSpanFolder {
    span: Span,
}

impl WithSpanFolder {
    fn new(span: Span) -> Self {
        Self { span }
    }
}

impl Fold for WithSpanFolder {
    fn fold_span(&mut self, _: Span) -> Span {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use rstest::rstest;
    use syn::parse_quote;

    use super::*;

    #[rstest]
    #[case(parse_quote!(Foo), false)]
    #[case(parse_quote!(T), true)]
    #[case(parse_quote!((T)), true)]
    #[case(parse_quote!((Foo, T)), true)]
    #[case(parse_quote!([T]), true)]
    #[case(parse_quote!([T; 1]), true)]
    #[case(parse_quote!(fn(T) -> Foo), true)]
    #[case(parse_quote!(fn(Foo) -> T), true)]
    #[case(parse_quote!(*const T), true)]
    #[case(parse_quote!(*mut T), true)]
    #[case(parse_quote!(&T), true)]
    #[case(parse_quote!(&mut T), true)]
    #[case(parse_quote!(Foo::T), false)]
    #[case(parse_quote!(::T::Foo), false)]
    #[case(parse_quote!(Foo<Bar>), false)]
    #[case(parse_quote!(Foo<T>), true)]
    #[case(parse_quote!(Foo<Bar<T>>), true)]
    #[case(parse_quote!(T<Foo>), true)]
    #[case(parse_quote!(T::Foo<Bar>), true)]
    #[case(parse_quote!(<T as Foo>::Bar<Baz>), true)]
    #[case(parse_quote!(Foo<Bar, T>), true)]
    #[case(parse_quote!(Foo<'T>), false)]
    fn test_dependency_on_type(#[case] ty: Type, #[case] expected_result: bool) {
        let type_ident = Ident::new("T", Span::call_site());

        assert_eq!(
            ty.dependency_on_type(&type_ident).is_some(),
            expected_result
        );
    }

    #[rstest]
    #[case(parse_quote!(Foo), parse_quote!(Foo))]
    #[case(parse_quote!(T), parse_quote!(A))]
    #[case(parse_quote!((T)), parse_quote!((A)))]
    #[case(parse_quote!((Foo, T)), parse_quote!((Foo, A)))]
    #[case(parse_quote!([T]), parse_quote!([A]))]
    #[case(parse_quote!([T; 1]), parse_quote!([A; 1]))]
    #[case(parse_quote!(fn(T) -> Foo), parse_quote!(fn(A) -> Foo))]
    #[case(parse_quote!(fn(Foo) -> T), parse_quote!(fn(Foo) -> A))]
    #[case(parse_quote!(*const T), parse_quote!(*const A))]
    #[case(parse_quote!(*mut T), parse_quote!(*mut A))]
    #[case(parse_quote!(&T), parse_quote!(&A))]
    #[case(parse_quote!(&mut T), parse_quote!(&mut A))]
    #[case(parse_quote!(Foo::T), parse_quote!(Foo::T))]
    #[case(parse_quote!(::T::Foo), parse_quote!(::T::Foo))]
    #[case(parse_quote!(Foo<Bar>), parse_quote!(Foo<Bar>))]
    #[case(parse_quote!(Foo<T>), parse_quote!(Foo<A>))]
    #[case(parse_quote!(Foo<Bar<T>>), parse_quote!(Foo<Bar<A>>))]
    #[case(parse_quote!(T<Foo>), parse_quote!(A<Foo>))]
    #[case(parse_quote!(T::Foo<Bar>), parse_quote!(A::Foo<Bar>))]
    #[case(parse_quote!(<T as Foo>::Bar<Baz>), parse_quote!(<A as Foo>::Bar<Baz>))]
    #[case(parse_quote!(Foo<Bar, T>), parse_quote!(Foo<Bar, A>))]
    #[case(parse_quote!(Foo<'T>), parse_quote!(Foo<'T>))]
    fn test_subs_type(#[case] ty: Type, #[case] expected_result: Type) {
        let type_ident = Ident::new("T", Span::call_site());
        let subs_type_ident = Ident::new("A", Span::call_site());

        assert_eq!(ty.subs_type(&type_ident, &subs_type_ident), expected_result);
    }
}
