use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(params(N))]
struct Test<T, const N: usize>(T);

fn main() {}
