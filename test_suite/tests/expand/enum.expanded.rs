use funcmap::{FuncMap, TryFuncMap};
enum Test<T> {
    UnitVariant,
    TupleVariant(T, i32),
    StructVariant {
        mapped_field: T,
        unmapped_field: i32,
    },
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
impl<A, B> ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A> {
    type Output = Test<B>;
    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: ::core::ops::FnMut(A) -> B,
    {
        match self {
            Self::UnitVariant {} => Self::Output::UnitVariant {},
            Self::TupleVariant {
                0: field_0,
                1: field_1,
            } => Self::Output::TupleVariant {
                0: f(field_0),
                1: field_1,
            },
            Self::StructVariant {
                mapped_field: field_mapped_field,
                unmapped_field: field_unmapped_field,
            } => Self::Output::StructVariant {
                mapped_field: f(field_mapped_field),
                unmapped_field: field_unmapped_field,
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
impl<A, B> ::funcmap::TryFuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A> {
    fn try_func_map<E, F>(self, mut f: F) -> ::core::result::Result<Self::Output, E>
    where
        F: ::core::ops::FnMut(A) -> ::core::result::Result<B, E>,
    {
        ::core::result::Result::Ok(match self {
            Self::UnitVariant {} => Self::Output::UnitVariant {},
            Self::TupleVariant {
                0: field_0,
                1: field_1,
            } => Self::Output::TupleVariant {
                0: f(field_0)?,
                1: field_1,
            },
            Self::StructVariant {
                mapped_field: field_mapped_field,
                unmapped_field: field_unmapped_field,
            } => Self::Output::StructVariant {
                mapped_field: f(field_mapped_field)?,
                unmapped_field: field_unmapped_field,
            },
        })
    }
}
