use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(params(T, T))]
struct Test<T>(T);

fn main() {}
