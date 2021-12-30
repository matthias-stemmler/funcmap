use funcmap::FuncMap;
struct Test<S, T>(S, T, i32);
#[allow(bare_trait_objects)]
#[allow(non_camel_case_types)]
#[automatically_derived]
impl<A, B, T> ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A, T> {
    type Output = Test<B, T>;
    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
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
#[allow(bare_trait_objects)]
#[allow(non_camel_case_types)]
#[automatically_derived]
impl<S, A, B> ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<1usize>> for Test<S, A> {
    type Output = Test<S, B>;
    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
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
