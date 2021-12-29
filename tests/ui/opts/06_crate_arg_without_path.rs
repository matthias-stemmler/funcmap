use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(crate =)]
struct Test<T>(T);

fn main() {}
