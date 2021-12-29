use funcmap::FuncMap;

macro_rules! test_type {
    () => {
        T
    };
}

#[derive(FuncMap)]
struct Test<T>(test_type!());

fn main() {}
