use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(params('a))]
struct Test<T>(T);

fn main() {}
