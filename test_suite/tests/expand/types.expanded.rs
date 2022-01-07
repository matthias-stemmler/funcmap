use funcmap::FuncMap;
struct Test<T> {
    tuple: (T, i32),
    array: [T; 1],
    nested: Foo<Bar<T>>,
    repeated: Foo<T, T>,
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
impl<A, B> ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A>
where
    Foo<Bar<A>>:
        ::funcmap::FuncMap<Bar<A>, Bar<B>, ::funcmap::TypeParam<0usize>, Output = Foo<Bar<B>>>,
    Bar<A>: ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<0usize>, Output = Bar<B>>,
    Foo<A, A>: ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<0usize>, Output = Foo<B, A>>,
    Foo<B, A>: ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<1usize>, Output = Foo<B, B>>,
{
    type Output = Test<B>;
    fn try_func_map<F, E>(self, mut f: F) -> ::core::result::Result<Self::Output, E>
    where
        F: ::core::ops::FnMut(A) -> ::core::result::Result<B, E>,
    {
        ::core::result::Result::Ok(match self {
            Self {
                tuple: field_tuple,
                array: field_array,
                nested: field_nested,
                repeated: field_repeated,
            } => Self::Output {
                tuple: (f(field_tuple.0)?, field_tuple.1),
                array: ::funcmap::FuncMap::try_func_map(field_array, |value| {
                    ::core::result::Result::Ok(f(value)?)
                })?,
                nested: ::funcmap::FuncMap::<_, _, ::funcmap::TypeParam<0usize>>::try_func_map(
                    field_nested,
                    |value| {
                        ::core::result::Result::Ok(::funcmap::FuncMap::<
                            _,
                            _,
                            ::funcmap::TypeParam<0usize>,
                        >::try_func_map(
                            value,
                            |value| ::core::result::Result::Ok(f(value)?),
                        )?)
                    },
                )?,
                repeated: ::funcmap::FuncMap::<_, _, ::funcmap::TypeParam<1usize>>::try_func_map(
                    ::funcmap::FuncMap::<_, _, ::funcmap::TypeParam<0usize>>::try_func_map(
                        field_repeated,
                        |value| ::core::result::Result::Ok(f(value)?),
                    )?,
                    |value| ::core::result::Result::Ok(f(value)?),
                )?,
            },
        })
    }
}
