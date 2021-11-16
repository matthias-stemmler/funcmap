use proc_macro_error::abort;
use std::collections::{HashMap, HashSet};
use syn::{
    punctuated::Punctuated, token::Add, BoundLifetimes, Lifetime, PredicateLifetime, PredicateType,
    Type, TypeParamBound, WhereClause, WherePredicate,
};

#[derive(Debug, Default)]
pub struct UniqueTypeBounds(HashSet<TypeParamBound>);

impl UniqueTypeBounds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, bound: TypeParamBound) {
        self.0.insert(bound);
    }

    pub fn into_bounds(self) -> Punctuated<TypeParamBound, Add> {
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
pub struct UniqueLifetimeBounds(HashSet<Lifetime>);

impl UniqueLifetimeBounds {
    pub fn into_bounds(self) -> Punctuated<Lifetime, Add> {
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
    for_types: HashMap<TypePredicateLhs, UniqueTypeBounds>,
    for_lifetimes: HashMap<Lifetime, UniqueLifetimeBounds>,
}

impl UniquePredicates {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, predicate: WherePredicate) {
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

            WherePredicate::Eq(..) => abort!(
                predicate,
                "equality predicates in `where` clauses are not supported"
            ),
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = WherePredicate> {
        let for_types = self.for_types.into_iter().map(|(lhs, rhs)| {
            WherePredicate::Type(PredicateType {
                lifetimes: lhs.lifetimes,
                bounded_ty: lhs.bounded_ty,
                colon_token: Default::default(),
                bounds: rhs.into_bounds(),
            })
        });

        let for_lifetimes = self.for_lifetimes.into_iter().map(|(lhs, rhs)| {
            WherePredicate::Lifetime(PredicateLifetime {
                lifetime: lhs,
                colon_token: Default::default(),
                bounds: rhs.into_bounds(),
            })
        });

        for_types.chain(for_lifetimes)
    }

    pub fn into_where_clause(self) -> WhereClause {
        WhereClause {
            where_token: Default::default(),
            predicates: self.into_iter().collect(),
        }
    }
}

impl Extend<WherePredicate> for UniquePredicates {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = WherePredicate>,
    {
        for predicate in iter {
            self.add(predicate);
        }
    }
}
