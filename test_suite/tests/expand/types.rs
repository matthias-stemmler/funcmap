use funcmap::{FuncMap, TryFuncMap};

#[derive(FuncMap, TryFuncMap)]
struct Test<T> {
    tuple: (T, i32),
    array: [T; 1],
    nested: Foo<Bar<T>>,
    repeated: Foo<T, T>,
}
