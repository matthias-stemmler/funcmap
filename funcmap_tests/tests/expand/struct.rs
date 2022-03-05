use funcmap::{FuncMap, TryFuncMap};

#[derive(FuncMap, TryFuncMap)]
struct Test<T> {
    mapped_field: T,
    unmapped_field: i32,
}
