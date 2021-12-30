use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(crate = "fake_funcmap::")]
struct Test<T>(T);

fn main() {}
