use funcmap::{FuncMap, TryFuncMap};
struct Test<T> {
    mapped_field: T,
    unmapped_field: i32,
}
#[allow(absolute_paths_not_starting_with_crate)]
#[allow(bare_trait_objects)]
#[allow(deprecated)]
#[allow(drop_bounds)]
#[allow(dyn_drop)]
#[allow(non_camel_case_types)]
#[allow(trivial_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::disallowed_method)]
#[allow(clippy::disallowed_type)]
#[automatically_derived]
impl<T> ::funcmap::FuncMap_cannot_be_derived_for_types_implementing_Drop for Test<T> {}
#[allow(absolute_paths_not_starting_with_crate)]
#[allow(bare_trait_objects)]
#[allow(deprecated)]
#[allow(drop_bounds)]
#[allow(dyn_drop)]
#[allow(non_camel_case_types)]
#[allow(trivial_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::disallowed_method)]
#[allow(clippy::disallowed_type)]
#[automatically_derived]
impl<A, B> ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A>
where
    i32: ::core::marker::Sized,
{
    type Output = Test<B>;
    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: ::core::ops::FnMut(A) -> B,
    {
        match self {
            Self {
                mapped_field: field_mapped_field,
                unmapped_field: field_unmapped_field,
            } => {
                Self::Output {
                    mapped_field: f(field_mapped_field),
                    unmapped_field: field_unmapped_field,
                }
            }
        }
    }
}
#[allow(absolute_paths_not_starting_with_crate)]
#[allow(bare_trait_objects)]
#[allow(deprecated)]
#[allow(drop_bounds)]
#[allow(dyn_drop)]
#[allow(non_camel_case_types)]
#[allow(trivial_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::disallowed_method)]
#[allow(clippy::disallowed_type)]
#[automatically_derived]
impl<T> ::funcmap::TryFuncMap_cannot_be_derived_for_types_implementing_Drop for Test<T> {}
#[allow(absolute_paths_not_starting_with_crate)]
#[allow(bare_trait_objects)]
#[allow(deprecated)]
#[allow(drop_bounds)]
#[allow(dyn_drop)]
#[allow(non_camel_case_types)]
#[allow(trivial_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::disallowed_method)]
#[allow(clippy::disallowed_type)]
#[automatically_derived]
impl<A, B> ::funcmap::TryFuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A>
where
    i32: ::core::marker::Sized,
{
    type Output = Test<B>;
    fn try_func_map<E, F>(self, mut f: F) -> ::core::result::Result<Self::Output, E>
    where
        F: ::core::ops::FnMut(A) -> ::core::result::Result<B, E>,
    {
        ::core::result::Result::Ok(
            match self {
                Self {
                    mapped_field: field_mapped_field,
                    unmapped_field: field_unmapped_field,
                } => {
                    Self::Output {
                        mapped_field: f(field_mapped_field)?,
                        unmapped_field: field_unmapped_field,
                    }
                }
            },
        )
    }
}
