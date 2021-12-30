use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(unknown)]
struct Test<T>(T);

fn main() {}
