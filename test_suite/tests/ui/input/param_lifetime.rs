use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(params('a))]
struct Test<'a, T>(&'a (), T);

fn main() {}
