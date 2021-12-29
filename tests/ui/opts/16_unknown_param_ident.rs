use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(params(S))]
struct Test<T>(T);

fn main() {}
