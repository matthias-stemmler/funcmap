use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(crate = "fake_funcmap_1")]
#[funcmap(crate = "fake_funcmap_2")]
struct Test<T>(T);

fn main() {}
