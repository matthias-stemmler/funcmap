use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<T> {
    mapped_field: T,
    unmapped_field: i32,
}
