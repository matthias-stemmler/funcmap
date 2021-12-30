use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<T> {
    tuple: (T, i32),
    array: [T; 1],
    nested: Foo<Bar<T>>,
    repeated: Foo<T, T>,
}
