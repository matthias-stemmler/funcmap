use funcmap::{FuncMap, TryFuncMap};
struct Test<S, T>(S, T, i32);
#[allow(absolute_paths_not_starting_with_crate)]
#[allow(bare_trait_objects)]
#[allow(deprecated)]
#[allow(drop_bounds)]
#[allow(dyn_drop)]
#[allow(keyword_idents)]
#[allow(non_camel_case_types)]
#[allow(trivial_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::disallowed_method)]
#[allow(clippy::disallowed_type)]
#[automatically_derived]
impl<A, B, T> ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A, T>
where
    T: ::core::marker::Sized,
    i32: ::core::marker::Sized,
{
    type Output = Test<B, T>;
    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: ::core::ops::FnMut(A) -> B,
    {
        match self {
            Self {
                0: field_0,
                1: field_1,
                2: field_2,
            } => Self::Output {
                0: f(field_0),
                1: field_1,
                2: field_2,
            },
        }
    }
}
#[allow(absolute_paths_not_starting_with_crate)]
#[allow(bare_trait_objects)]
#[allow(deprecated)]
#[allow(drop_bounds)]
#[allow(dyn_drop)]
#[allow(keyword_idents)]
#[allow(non_camel_case_types)]
#[allow(trivial_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::disallowed_method)]
#[allow(clippy::disallowed_type)]
#[automatically_derived]
impl<S, A, B> ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<1usize>> for Test<S, A>
where
    S: ::core::marker::Sized,
    i32: ::core::marker::Sized,
{
    type Output = Test<S, B>;
    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: ::core::ops::FnMut(A) -> B,
    {
        match self {
            Self {
                0: field_0,
                1: field_1,
                2: field_2,
            } => Self::Output {
                0: field_0,
                1: f(field_1),
                2: field_2,
            },
        }
    }
}
#[allow(absolute_paths_not_starting_with_crate)]
#[allow(bare_trait_objects)]
#[allow(deprecated)]
#[allow(drop_bounds)]
#[allow(dyn_drop)]
#[allow(keyword_idents)]
#[allow(non_camel_case_types)]
#[allow(trivial_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::disallowed_method)]
#[allow(clippy::disallowed_type)]
#[automatically_derived]
impl<A, B, T> ::funcmap::TryFuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A, T>
where
    T: ::core::marker::Sized,
    i32: ::core::marker::Sized,
{
    type Output = Test<B, T>;
    fn try_func_map<E, F>(self, mut f: F) -> ::core::result::Result<Self::Output, E>
    where
        F: ::core::ops::FnMut(A) -> ::core::result::Result<B, E>,
    {
        ::core::result::Result::Ok(match self {
            Self {
                0: field_0,
                1: field_1,
                2: field_2,
            } => Self::Output {
                0: f(field_0)?,
                1: field_1,
                2: field_2,
            },
        })
    }
}
#[allow(absolute_paths_not_starting_with_crate)]
#[allow(bare_trait_objects)]
#[allow(deprecated)]
#[allow(drop_bounds)]
#[allow(dyn_drop)]
#[allow(keyword_idents)]
#[allow(non_camel_case_types)]
#[allow(trivial_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::disallowed_method)]
#[allow(clippy::disallowed_type)]
#[automatically_derived]
impl<S, A, B> ::funcmap::TryFuncMap<A, B, ::funcmap::TypeParam<1usize>> for Test<S, A>
where
    S: ::core::marker::Sized,
    i32: ::core::marker::Sized,
{
    type Output = Test<S, B>;
    fn try_func_map<E, F>(self, mut f: F) -> ::core::result::Result<Self::Output, E>
    where
        F: ::core::ops::FnMut(A) -> ::core::result::Result<B, E>,
    {
        ::core::result::Result::Ok(match self {
            Self {
                0: field_0,
                1: field_1,
                2: field_2,
            } => Self::Output {
                0: field_0,
                1: f(field_1)?,
                2: field_2,
            },
        })
    }
}
