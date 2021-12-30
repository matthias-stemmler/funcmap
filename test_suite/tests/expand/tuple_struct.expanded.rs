use funcmap::FuncMap;
struct Test<T>(T, i32);
#[allow(bare_trait_objects)]
#[allow(non_camel_case_types)]
#[automatically_derived]
impl<A, B> ::funcmap::FuncMap<A, B, ::funcmap::TypeParam<0usize>> for Test<A> {
    type Output = Test<B>;
    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        match self {
            Self {
                0: field_0,
                1: field_1,
            } => Self::Output {
                0: f(field_0),
                1: field_1,
            },
        }
    }
}
