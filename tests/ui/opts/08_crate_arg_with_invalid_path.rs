use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(crate = "not a path")]
struct Test<T>(T);

fn main() {}
