use funcmap::{FuncMap, TryFuncMap};

#[derive(FuncMap, TryFuncMap)]
enum Test<T> {
    UnitVariant,
    TupleVariant(T, i32),
    StructVariant {
        mapped_field: T,
        unmapped_field: i32,
    },
}
