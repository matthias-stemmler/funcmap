use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<T>(dyn Fn(T));

fn main() {}
