use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<T>(fn(T));

fn main() {}
