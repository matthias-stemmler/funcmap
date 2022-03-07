//! Additional functionality for types in the [`syn`] crate

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::fold::{self, Fold};
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{
    ConstParam, GenericArgument, GenericParam, LifetimeDef, PathSegment, PredicateType, TraitBound,
    TraitBoundModifier, Type, TypeParam, TypeParamBound, TypePath, WherePredicate,
};

/// Extension trait for determining the dependency of an AST node on a type
pub(crate) trait DependencyOnType {
    /// Returns the dependency of `self` on a type named `type_ident`, if any
    ///
    /// The returned [`Ident`] may differ from `type_ident` only in its
    /// [`Span`](proc_macro2::Span).
    ///
    /// Note that macros in type position are always considered to be
    /// independent of the given type.
    fn dependency_on_type(&self, type_ident: &Ident) -> Option<&Ident>;
}

impl DependencyOnType for Type {
    fn dependency_on_type(&self, type_ident: &Ident) -> Option<&Ident> {
        let mut visitor = DependencyOnTypeVisitor::new(type_ident);
        visitor.visit_type(self);
        visitor.into_dependency()
    }
}

/// Type implementing [`Visit`] for
/// [`dependency_on_type`](DependencyOnType::dependency_on_type)
#[derive(Debug)]
struct DependencyOnTypeVisitor<'ast, 'a> {
    dependency: Option<&'ast Ident>,
    type_ident: &'a Ident,
}

impl<'ast, 'a> DependencyOnTypeVisitor<'ast, 'a> {
    fn new(type_ident: &'a Ident) -> Self {
        Self {
            dependency: None,
            type_ident,
        }
    }

    fn into_dependency(self) -> Option<&'ast Ident> {
        self.dependency
    }
}

impl<'ast> Visit<'ast> for DependencyOnTypeVisitor<'ast, '_> {
    fn visit_type(&mut self, ty: &'ast Type) {
        if self.dependency.is_some() {
            return;
        }

        match ty {
            Type::Path(TypePath { qself: None, path }) if path.leading_colon.is_none() => {
                match path.segments.first() {
                    Some(PathSegment { ident, .. }) if ident == self.type_ident => {
                        self.dependency = Some(ident);
                    }
                    _ => visit::visit_type(self, ty),
                }
            }
            _ => visit::visit_type(self, ty),
        }
    }
}

/// Extension trait for substituting one type with another in an AST node
pub(crate) trait SubsType {
    /// Substitutes the type named `type_ident` with `subs_ident` within `self`
    fn subs_type(self, type_ident: &Ident, subs_ident: &Ident) -> Self;
}

impl SubsType for Type {
    fn subs_type(self, type_ident: &Ident, subs_ident: &Ident) -> Self {
        let mut folder = SubsTypeFolder::new(type_ident, subs_ident);
        folder.fold_type(self)
    }
}

impl SubsType for TraitBound {
    fn subs_type(self, type_ident: &Ident, subs_ident: &Ident) -> Self {
        let mut folder = SubsTypeFolder::new(type_ident, subs_ident);
        folder.fold_trait_bound(self)
    }
}

impl SubsType for WherePredicate {
    fn subs_type(self, type_ident: &Ident, subs_ident: &Ident) -> Self {
        let mut folder = SubsTypeFolder::new(type_ident, subs_ident);
        folder.fold_where_predicate(self)
    }
}

/// Type implementing [`Fold`] for [`subs_type`](SubsType::subs_type)
struct SubsTypeFolder<'a> {
    type_ident: &'a Ident,
    subs_ident: &'a Ident,
}

impl<'a> SubsTypeFolder<'a> {
    fn new(type_ident: &'a Ident, subs_ident: &'a Ident) -> Self {
        Self {
            type_ident,
            subs_ident,
        }
    }
}

impl<'a> Fold for SubsTypeFolder<'a> {
    fn fold_type(&mut self, mut ty: Type) -> Type {
        match &mut ty {
            Type::Path(TypePath { qself: None, path }) if path.leading_colon.is_none() => {
                match path.segments.first_mut() {
                    Some(PathSegment { ident, .. }) if ident == self.type_ident => {
                        *ident = self.subs_ident.clone();
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        fold::fold_type(self, ty)
    }
}

/// Extension trait for converting an AST node into a [`GenericArgument`]
pub(crate) trait IntoGenericArgument {
    /// Converts `self` into a [`GenericArgument`]
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

/// Extension trait for converting an [`Ident`] into a [`Type`]
pub(crate) trait IntoType {
    /// Converts `self` into a [`Type`]
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

/// Extension trait for determining whether an AST node is "typish", i.e. if it
/// may be syntactically indistinguishable from a type
pub(crate) trait IsTypish {
    /// Returns `true` if `self` is a type or something that may be
    /// syntactically indistinguishable from a type such as a const parameter
    fn is_typish(&self) -> bool;
}

impl IsTypish for GenericArgument {
    fn is_typish(&self) -> bool {
        matches!(self, GenericArgument::Type(..) | GenericArgument::Const(..))
    }
}

impl IsTypish for GenericParam {
    fn is_typish(&self) -> bool {
        matches!(self, GenericParam::Type(..) | GenericParam::Const(..))
    }
}

/// Extension trait for interpolating types inside a `quote!` invocation if they
/// produce a non-empty [`TokenStream`]
pub(crate) trait ToNonEmptyTokens {
    /// Converts `self` into a non-empty [`TokenStream`] if possible
    ///
    /// Returns the result of [`ToTokens::to_token_stream`] if it is non-empty,
    /// or [`None`] otherwise.
    fn to_non_empty_token_stream(&self) -> Option<TokenStream>;
}

impl<T> ToNonEmptyTokens for T
where
    T: ?Sized + ToTokens,
{
    fn to_non_empty_token_stream(&self) -> Option<TokenStream> {
        match self.to_token_stream() {
            tokens if !tokens.is_empty() => Some(tokens),
            _ => None,
        }
    }
}

/// Extension trait for removing attributes from an AST node
pub(crate) trait WithoutAttrs {
    /// Returns `self` without attributes
    fn without_attrs(self) -> Self;
}

impl WithoutAttrs for ConstParam {
    fn without_attrs(self) -> Self {
        WithoutAttrsFolder.fold_const_param(self)
    }
}

impl WithoutAttrs for GenericParam {
    fn without_attrs(self) -> Self {
        WithoutAttrsFolder.fold_generic_param(self)
    }
}

impl WithoutAttrs for LifetimeDef {
    fn without_attrs(self) -> Self {
        WithoutAttrsFolder.fold_lifetime_def(self)
    }
}

impl WithoutAttrs for TraitBound {
    fn without_attrs(self) -> Self {
        WithoutAttrsFolder.fold_trait_bound(self)
    }
}

impl WithoutAttrs for WherePredicate {
    fn without_attrs(self) -> Self {
        WithoutAttrsFolder.fold_where_predicate(self)
    }
}

/// Type implementing [`Fold`] for
/// [`without_attrs`](WithoutAttrs::without_attrs)
#[derive(Debug)]
struct WithoutAttrsFolder;

impl Fold for WithoutAttrsFolder {
    fn fold_const_param(&mut self, const_param: ConstParam) -> ConstParam {
        ConstParam {
            attrs: Vec::new(),
            ..const_param
        }
    }

    fn fold_lifetime_def(&mut self, lifetime_def: LifetimeDef) -> LifetimeDef {
        LifetimeDef {
            attrs: Vec::new(),
            ..lifetime_def
        }
    }

    fn fold_type_param(&mut self, type_param: TypeParam) -> TypeParam {
        TypeParam {
            attrs: Vec::new(),
            ..type_param
        }
    }
}

/// Extension trait for removing the default value from a generic parameter
pub(crate) trait WithoutDefault {
    /// Returns `self` without a default value
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

impl WithoutDefault for GenericParam {
    fn without_default(self) -> Self {
        match self {
            Self::Type(type_param) => Self::Type(type_param.without_default()),
            Self::Const(const_param) => Self::Const(const_param.without_default()),
            Self::Lifetime(..) => self,
        }
    }
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

/// Extension trait for removing bounds such as `?Sized` from an AST node
pub(crate) trait WithoutMaybeBounds {
    /// Returns `self` without maybe bounds
    fn without_maybe_bounds(self) -> Self;
}

impl<P> WithoutMaybeBounds for Punctuated<TypeParamBound, P>
where
    P: Default,
{
    fn without_maybe_bounds(self) -> Self {
        self.into_iter()
            .filter(|bound| {
                !matches!(
                    bound,
                    TypeParamBound::Trait(TraitBound {
                        modifier: TraitBoundModifier::Maybe(..),
                        ..
                    })
                )
            })
            .collect()
    }
}

impl WithoutMaybeBounds for PredicateType {
    fn without_maybe_bounds(self) -> Self {
        Self {
            bounds: self.bounds.without_maybe_bounds(),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proc_macro2::Span;
    use quote::quote;
    use syn::{parse_quote, Token};

    mod type_dependency {
        use super::*;

        macro_rules! type_dependency_test {
            ($name:ident: $src_type:ty => $dst_type:ty) => {
                mod $name {
                    use super::*;

                    #[test]
                    fn dependency_on_type_is_some_if_dependent() {
                        let ty: Type = parse_quote!($src_type);
                        let type_ident = Ident::new("A", Span::call_site());

                        assert_eq!(ty.dependency_on_type(&type_ident), Some(&type_ident));
                    }

                    #[test]
                    fn subs_type_substitutes_type_if_dependent() {
                        let src_type: Type = parse_quote!($src_type);
                        let dst_type: Type = parse_quote!($dst_type);

                        let type_ident = Ident::new("A", Span::call_site());
                        let subs_type_ident = Ident::new("B", Span::call_site());

                        assert_eq!(src_type.subs_type(&type_ident, &subs_type_ident), dst_type);
                    }
                }
            };

            ($name:ident: $type:ty) => {
                mod $name {
                    use super::*;

                    #[test]
                    fn dependency_on_type_is_none_if_independent() {
                        let ty: Type = parse_quote!($type);
                        let type_ident = Ident::new("A", Span::call_site());

                        assert!(ty.dependency_on_type(&type_ident).is_none());
                    }

                    #[test]
                    fn subs_type_does_not_substitute_if_independent() {
                        let ty: Type = parse_quote!($type);

                        let type_ident = Ident::new("A", Span::call_site());
                        let subs_type_ident = Ident::new("B", Span::call_site());

                        assert_eq!(ty.clone().subs_type(&type_ident, &subs_type_ident), ty);
                    }
                }
            };
        }

        macro_rules! type_dependency_tests {
            ($($name:ident: $($type:ty)=>*,)*) => {
                $(type_dependency_test!($name: $($type)=>*);)*
            };
        }

        type_dependency_tests! {
            array_dep: [A; 1] => [B; 1],
            array_ind: [Foo; 1],
            fn_dep: fn(A) -> A => fn(B) -> B,
            fn_ind: fn(Foo) -> Foo,
            impl_trait: impl A,
            impl_trait_generic_dep: impl Trait<A> => impl Trait<B>,
            impl_trait_generic_ind: impl Trait<Foo>,
            impl_trait_assoc_dep: impl Trait<Assoc = A> => impl Trait<Assoc = B>,
            impl_trait_assoc_ind: impl Trait<Assoc = Foo>,
            infer: _,
            macro_name: A!(),
            macro_arg: test_macro!(A),
            never: !,
            paren_dep: (A) => (B),
            paren_ind: (Foo),
            path_ident_dep: A => B,
            path_ident_ind: Foo,
            path_ident_with_arg_dep: A<Bar> => B<Bar>,
            path_ident_with_arg_ind: Foo<Bar>,
            path_with_sub_dep: A::Bar => B::Bar,
            path_with_sub_ind: Foo::Bar,
            path_leading_colon: ::A,
            path_with_super: Foo::A,
            path_arg_dep: Foo<A> => Foo<B>,
            path_arg_ind: Foo<Bar>,
            path_nested_arg_dep: Foo<Bar<A>> => Foo<Bar<B>>,
            path_nested_arg_ind: Foo<Bar<Baz>>,
            path_qself_type_dep: <A as Bar>::Baz => <B as Bar>::Baz,
            path_qself_type_ind: <Foo as Bar>::Baz,
            path_qself_trait: <Foo as A>::Bar,
            path_with_qself: <Foo as Bar>::A,
            path_lifetime: Foo<'A>,
            const_ptr_dep: *const A => *const B,
            const_ptr_ind: *const Foo,
            ptr_mut_dep: *mut A => *mut B,
            ptr_mut_ind: *mut Foo,
            ref_dep: &A => &B,
            ref_ind: &Foo,
            ref_mut_dep: &mut A => &mut B,
            ref_mut_ind: &mut Foo,
            slice_dep: [A] => [B],
            slice_ind: [Foo],
            dyn_trait: dyn A,
            dyn_trait_generic_dep: dyn Trait<A> => dyn Trait<B>,
            dyn_trait_generic_ind: dyn Trait<Foo>,
            dyn_trait_assoc_dep: dyn Trait<Assoc = A> => dyn Trait<Assoc = B>,
            dyn_trait_assoc_ind: dyn Trait<Assoc = Foo>,
            tuple_dep: (Foo, A) => (Foo, B),
            tuple_ind: (Foo, Bar),
        }
    }

    #[test]
    fn into_generic_argument_converts_type_param_into_type_argument() {
        let generic_param: GenericParam = parse_quote!(T);
        assert_eq!(generic_param.into_generic_argument(), parse_quote!(T));
    }

    #[test]
    fn into_generic_argument_converts_lifetime_param_into_lifetime_argument() {
        let generic_param: GenericParam = parse_quote!('a);
        assert_eq!(generic_param.into_generic_argument(), parse_quote!('a));
    }

    #[test]
    fn into_generic_argument_converts_const_param_into_type_argument() {
        let generic_param: GenericParam = parse_quote!(const N: usize);
        assert_eq!(generic_param.into_generic_argument(), parse_quote!(N));
    }

    #[test]
    fn into_type_converts_ident_into_type() {
        let ident = Ident::new("T", Span::call_site());
        assert_eq!(ident.into_type(), parse_quote!(T));
    }

    #[test]
    fn is_typish_returns_true_for_generic_argument_looking_like_type() {
        let ty: GenericArgument = parse_quote!(T);
        assert!(ty.is_typish());
    }

    #[test]
    fn is_typish_returns_false_for_generic_argument_not_looking_like_type() {
        let ty: GenericArgument = parse_quote!('a);
        assert!(!ty.is_typish());
    }

    #[test]
    fn is_typish_returns_true_for_generic_parameter_looking_like_type() {
        let ty: GenericParam = parse_quote!(T);
        assert!(ty.is_typish());
    }

    #[test]
    fn is_typish_returns_false_for_generic_parameter_not_looking_like_type() {
        let ty: GenericParam = parse_quote!('a);
        assert!(!ty.is_typish());
    }

    #[test]
    fn to_non_empty_token_stream_returns_some_if_token_stream_is_not_empty() {
        let tokens = quote!(X);
        assert!(tokens.to_non_empty_token_stream().is_some());
    }

    #[test]
    fn to_non_empty_token_stream_returns_none_if_token_stream_is_empty() {
        let tokens = quote!();
        assert!(tokens.to_non_empty_token_stream().is_none());
    }

    #[test]
    fn without_attrs_removes_attributes_from_const_param() {
        let const_param: GenericParam = parse_quote!(#[attr] const N: usize = 42);

        assert_eq!(
            const_param.without_attrs(),
            parse_quote!(const N: usize = 42)
        );
    }

    #[test]
    fn without_attrs_removes_attributes_from_lifetime_param() {
        let lifetime_param: GenericParam = parse_quote!(#[attr] 'a: 'b);
        assert_eq!(lifetime_param.without_attrs(), parse_quote!('a: 'b));
    }

    #[test]
    fn without_attrs_removes_attributes_from_type_param() {
        let type_param: GenericParam = parse_quote!(
            #[attr]
            T: Trait = Test
        );

        assert_eq!(type_param.without_attrs(), parse_quote!(T: Trait = Test));
    }

    #[test]
    fn without_default_removes_default_from_const_param() {
        let const_param: GenericParam = parse_quote!(#[attr] const N: usize = 42);

        assert_eq!(
            const_param.without_default(),
            parse_quote!(#[attr] const N: usize)
        );
    }

    #[test]
    fn without_default_removes_default_from_type_param() {
        let type_param: GenericParam = parse_quote!(
            #[attr]
            T: Trait = Test
        );

        assert_eq!(
            type_param.without_default(),
            parse_quote!(
                #[attr]
                T: Trait
            )
        );
    }

    #[test]
    fn without_maybe_bounds_removes_maybe_bounds_from_sequence_of_type_param_bounds() {
        let bounds: Punctuated<TypeParamBound, Token![+]> = parse_quote!(?Sized + Trait + 'b);
        assert_eq!(bounds.without_maybe_bounds(), parse_quote!(Trait + 'b))
    }

    #[test]
    fn without_maybe_bounds_removes_maybe_bounds_from_type_predicate() {
        let type_predicate: WherePredicate = parse_quote!(for<'a> Type: ?Sized + Trait + 'c);

        assert_eq!(
            match type_predicate {
                WherePredicate::Type(predicate_type) =>
                    WherePredicate::Type(predicate_type.without_maybe_bounds()),
                predicate => predicate,
            },
            parse_quote!(for<'a> Type: Trait + 'c),
        )
    }
}
