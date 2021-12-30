use funcmap::FuncMap;
enum Test<T> {
    UnitVariant,
    TupleVariant(T, i32),
    StructVariant {
        mapped_field: T,
        unmapped_field: i32,
    },
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
