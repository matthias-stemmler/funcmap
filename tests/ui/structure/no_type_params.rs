use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<'a, const N: usize>(&'a ());

fn main() {}
