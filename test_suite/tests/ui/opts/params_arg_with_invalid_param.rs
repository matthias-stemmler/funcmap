use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(params(!))]
struct Test<T>(T);

fn main() {}
