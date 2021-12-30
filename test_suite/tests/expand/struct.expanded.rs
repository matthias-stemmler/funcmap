use funcmap::FuncMap;
struct Test<T> {
    mapped_field: T,
    unmapped_field: i32,
}
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
                mapped_field: field_mapped_field,
                unmapped_field: field_unmapped_field,
            } => Self::Output {
                mapped_field: f(field_mapped_field),
                unmapped_field: field_unmapped_field,
            },
        }
    }
}
