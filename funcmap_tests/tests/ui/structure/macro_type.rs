use funcmap::FuncMap;

macro_rules! test_type {
    () => {
        T
    };
}

#[derive(FuncMap)]
struct Test<T>(test_type!());

// try to derive `Default` to make sure the error message is the same
#[derive(Default)]
struct TestDefault<T>(test_type!());

fn main() {}
