use funcmap::{FuncMap, TryFuncMap};

#[derive(FuncMap, TryFuncMap)]
struct Test<S, T>(S, T, i32);
