use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap]
#[funcmap(crate = "fake_funcmap_1", crate = "fake_funcmap_2")]
#[funcmap(params(T, T))]
struct Test<T>(T);

fn main() {}
