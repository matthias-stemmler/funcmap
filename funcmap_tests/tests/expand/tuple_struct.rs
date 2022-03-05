use funcmap::{FuncMap, TryFuncMap};

#[derive(FuncMap, TryFuncMap)]
struct Test<T>(T, i32);
