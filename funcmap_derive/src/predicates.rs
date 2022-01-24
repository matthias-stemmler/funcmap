//! Provides types dealing with type and lifetime predicates

use crate::result::Error;

use indexmap::{IndexMap, IndexSet};
use syn::{
    punctuated::Punctuated, BoundLifetimes, Lifetime, PredicateEq, PredicateLifetime,
    PredicateType, Token, Type, TypeParamBound, WhereClause, WherePredicate,
};

/// A set of unique type bounds
#[derive(Debug, Default)]
pub(crate) struct UniqueTypeBounds(IndexSet<TypeParamBound>);

impl UniqueTypeBounds {
    /// Creates an empty set of unique type bounds
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Adds the given type bound to this set
    pub(crate) fn add(&mut self, bound: TypeParamBound) {
        self.0.insert(bound);
    }

    /// Turn this set into a `+`-punctuated sequence
    pub(crate) fn into_bounds(self) -> Punctuated<TypeParamBound, Token![+]> {
        self.0.into_iter().collect()
    }
}

impl Extend<TypeParamBound> for UniqueTypeBounds {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = TypeParamBound>,
    {
        self.0.extend(iter);
    }
}

/// A set of unique lifetime bounds
#[derive(Debug, Default)]
struct UniqueLifetimeBounds(IndexSet<Lifetime>);

impl UniqueLifetimeBounds {
    /// Turns this set into a `+`-punctuated sequence
    fn into_bounds(self) -> Punctuated<Lifetime, Token![+]> {
        self.0.into_iter().collect()
    }
}

impl Extend<Lifetime> for UniqueLifetimeBounds {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Lifetime>,
    {
        self.0.extend(iter);
    }
}

/// The left-hand side of a type predicate
#[derive(Debug, Eq, Hash, PartialEq)]
struct TypePredicateLhs {
    lifetimes: Option<BoundLifetimes>,
    bounded_ty: Type,
}

/// A set of unique type or lifetime predicates
#[derive(Debug, Default)]
pub(crate) struct UniquePredicates {
    for_types: IndexMap<TypePredicateLhs, UniqueTypeBounds>,
    for_lifetimes: IndexMap<Lifetime, UniqueLifetimeBounds>,
}

impl UniquePredicates {
    /// Creates an empty set of unique type of lifetime predicates
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Adds the given predicate to this set
    pub(crate) fn add(&mut self, predicate: WherePredicate) -> Result<(), Error> {
        match predicate {
            WherePredicate::Type(predicate_type) => self
                .for_types
                .entry(TypePredicateLhs {
                    lifetimes: predicate_type.lifetimes,
                    bounded_ty: predicate_type.bounded_ty,
                })
                .or_default()
                .extend(predicate_type.bounds),

            WherePredicate::Lifetime(predicate_lifetime) => self
                .for_lifetimes
                .entry(predicate_lifetime.lifetime)
                .or_default()
                .extend(predicate_lifetime.bounds),

            WherePredicate::Eq(PredicateEq { eq_token, .. }) => {
                // currently unsupported in Rust
                // see https://github.com/rust-lang/rust/issues/20041
                return Err(syn::Error::new_spanned(
                    eq_token,
                    "equality constraints in `where` clauses are not supported",
                )
                .into());
            }
        }

        Ok(())
    }

    /// Turns this set into an iterator of predicates
    pub(crate) fn into_iter(self) -> impl Iterator<Item = WherePredicate> {
        let for_lifetimes = self.for_lifetimes.into_iter().filter_map(|(lhs, rhs)| {
            let bounds = rhs.into_bounds();

            (!bounds.is_empty()).then(|| {
                WherePredicate::Lifetime(PredicateLifetime {
                    lifetime: lhs,
                    colon_token: <Token![:]>::default(),
                    bounds,
                })
            })
        });

        let for_types = self.for_types.into_iter().filter_map(|(lhs, rhs)| {
            let bounds = rhs.into_bounds();

            (!bounds.is_empty()).then(|| {
                WherePredicate::Type(PredicateType {
                    lifetimes: lhs.lifetimes,
                    bounded_ty: lhs.bounded_ty,
                    colon_token: <Token![:]>::default(),
                    bounds,
                })
            })
        });

        for_lifetimes.chain(for_types)
    }

    /// Turns this set of predicates into a `where` clause
    pub(crate) fn into_where_clause(self) -> WhereClause {
        WhereClause {
            where_token: <Token![where]>::default(),
            predicates: self.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proc_macro2::{Ident, Span};
    use syn::parse_quote;

    #[test]
    fn unique_type_bounds_produces_bounds_in_insertion_order() {
        let mut unique_bounds = UniqueTypeBounds::new();
        unique_bounds.extend(
            [
                parse_quote!(TestTrait1),
                parse_quote!((TestTrait2)),
                parse_quote!(?Sized),
                parse_quote!(for<'a> TestTrait3<'a>),
                parse_quote!('b),
            ]
            .into_iter(),
        );

        assert_eq!(
            unique_bounds.into_bounds(),
            parse_quote!(TestTrait1 + (TestTrait2) + ?Sized + for<'a> TestTrait3<'a> + 'b)
        );
    }

    #[test]
    fn unique_type_bounds_eliminates_duplicates_regardless_of_span() {
        let trait_ident1 = Ident::new("TestTrait", Span::call_site());
        let trait_ident2 = Ident::new("TestTrait", Span::mixed_site());

        let mut unique_bounds = UniqueTypeBounds::new();
        unique_bounds
            .extend([parse_quote!(#trait_ident1), parse_quote!(#trait_ident2)].into_iter());

        assert_eq!(unique_bounds.into_bounds(), parse_quote!(TestTrait));
    }

    #[test]
    fn unique_predicates_produces_predicates_in_insertion_order_except_lifetimes_first() {
        let mut unique_predicates = UniquePredicates::new();
        for predicate in [
            parse_quote!(for<'a> TestType1: TestTrait1<'a>),
            parse_quote!('b: 'c),
            parse_quote!(for<'d> TestType2: TestTrait2<'d>),
            parse_quote!('e: 'f),
        ] {
            unique_predicates.add(predicate).unwrap();
        }

        assert_eq!(
            unique_predicates.into_where_clause(),
            parse_quote!(where 'b: 'c, 'e: 'f, for<'a> TestType1: TestTrait1<'a>, for<'d> TestType2: TestTrait2<'d>)
        );
    }

    #[test]
    fn unique_predicates_eliminates_duplicates_regardless_of_span() {
        let type_ident1 = Ident::new("TestType", Span::call_site());
        let type_ident2 = Ident::new("TestType", Span::mixed_site());
        let lifetime1 = Lifetime::new("'b", Span::call_site());
        let lifetime2 = Lifetime::new("'b", Span::mixed_site());

        let mut unique_predicates = UniquePredicates::new();
        for predicate in [
            parse_quote!(for<'a> #type_ident1: TestTrait<'a>),
            parse_quote!(#lifetime1: 'c),
            parse_quote!(for<'a> #type_ident2: TestTrait<'a>),
            parse_quote!(#lifetime2: 'c),
        ] {
            unique_predicates.add(predicate).unwrap();
        }

        assert_eq!(
            unique_predicates.into_where_clause(),
            parse_quote!(where 'b: 'c, for<'a> TestType: TestTrait<'a>)
        );
    }
}
