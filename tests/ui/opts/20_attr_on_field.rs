use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<T>(#[funcmap] T);

fn main() {}
