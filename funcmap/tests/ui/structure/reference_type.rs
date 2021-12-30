use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<'a, T>(&'a T);

fn main() {}
