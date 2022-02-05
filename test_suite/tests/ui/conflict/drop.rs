use funcmap::FuncMap;

#[derive(FuncMap)]
struct Test<S, T>(S, T);

impl<S, T> Drop for Test<S, T> {
    fn drop(&mut self) {}
}

fn main() {}
