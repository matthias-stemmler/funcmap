use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(params = T)]
struct Test<T>(T);

fn main() {}
