use crate::error::Error;

use indexmap::{IndexMap, IndexSet};
use syn::{
    punctuated::Punctuated, BoundLifetimes, Lifetime, PredicateEq, PredicateLifetime,
    PredicateType, Token, Type, TypeParamBound, WhereClause, WherePredicate,
};

#[derive(Debug, Default)]
pub struct UniqueTypeBounds(IndexSet<TypeParamBound>);

impl UniqueTypeBounds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, bound: TypeParamBound) {
        self.0.insert(bound);
    }

    pub fn into_bounds(self) -> Punctuated<TypeParamBound, Token![+]> {
        self.0.into_iter().collect()
    }
}

impl Extend<TypeParamBound> for UniqueTypeBounds {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = TypeParamBound>,
    {
        self.0.extend(iter)
    }
}

#[derive(Debug, Default)]
pub struct UniqueLifetimeBounds(IndexSet<Lifetime>);

impl UniqueLifetimeBounds {
    pub fn into_bounds(self) -> Punctuated<Lifetime, Token![+]> {
        self.0.into_iter().collect()
    }
}

impl Extend<Lifetime> for UniqueLifetimeBounds {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Lifetime>,
    {
        self.0.extend(iter)
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct TypePredicateLhs {
    lifetimes: Option<BoundLifetimes>,
    bounded_ty: Type,
}

#[derive(Debug, Default)]
pub struct UniquePredicates {
    for_types: IndexMap<TypePredicateLhs, UniqueTypeBounds>,
    for_lifetimes: IndexMap<Lifetime, UniqueLifetimeBounds>,
}

impl UniquePredicates {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, predicate: WherePredicate) -> Result<(), Error> {
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

    pub fn into_iter(self) -> impl Iterator<Item = WherePredicate> {
        let for_lifetimes = self.for_lifetimes.into_iter().filter_map(|(lhs, rhs)| {
            let bounds = rhs.into_bounds();

            match bounds.is_empty() {
                true => None,
                false => Some(WherePredicate::Lifetime(PredicateLifetime {
                    lifetime: lhs,
                    colon_token: Default::default(),
                    bounds,
                })),
            }
        });

        let for_types = self.for_types.into_iter().filter_map(|(lhs, rhs)| {
            let bounds = rhs.into_bounds();

            match bounds.is_empty() {
                true => None,
                false => Some(WherePredicate::Type(PredicateType {
                    lifetimes: lhs.lifetimes,
                    bounded_ty: lhs.bounded_ty,
                    colon_token: Default::default(),
                    bounds,
                })),
            }
        });

        for_lifetimes.chain(for_types)
    }

    pub fn into_where_clause(self) -> WhereClause {
        WhereClause {
            where_token: Default::default(),
            predicates: self.into_iter().collect(),
        }
    }
}
