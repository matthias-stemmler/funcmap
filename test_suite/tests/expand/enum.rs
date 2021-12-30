use funcmap::FuncMap;

#[derive(FuncMap)]
enum Test<T> {
    UnitVariant,
    TupleVariant(T, i32),
    StructVariant {
        mapped_field: T,
        unmapped_field: i32,
    },
}
