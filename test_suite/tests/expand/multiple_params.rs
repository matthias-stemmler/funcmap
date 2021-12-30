use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<S, T>(S, T, i32);
