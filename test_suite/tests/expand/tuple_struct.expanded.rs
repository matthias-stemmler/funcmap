use funcmap::FuncMap;
struct Test<T>(T, i32);
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
    fn try_func_map<E, F>(self, mut f: F) -> ::core::result::Result<Self::Output, E>
    where
        F: ::core::ops::FnMut(A) -> ::core::result::Result<B, E>,
    {
        ::core::result::Result::Ok(match self {
            Self {
                0: field_0,
                1: field_1,
            } => Self::Output {
                0: f(field_0)?,
                1: field_1,
            },
        })
    }
}
