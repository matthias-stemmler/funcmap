use funcmap::FuncMap;

#[derive(FuncMap)]
enum Test<T> {
    #[funcmap]
    TestVariant(T),
}

fn main() {}
