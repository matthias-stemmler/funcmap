use funcmap::FuncMap;

trait TestTrait {
    type Assoc;
}

#[derive(FuncMap)]
struct Test<T>(<T as TestTrait>::Assoc)
where
    T: TestTrait;

fn main() {}
