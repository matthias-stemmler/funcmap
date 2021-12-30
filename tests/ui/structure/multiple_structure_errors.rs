use funcmap::FuncMap;

macro_rules! test_type {
    () => {
        T
    };
}

trait TestTrait {
    type Assoc;
}

#[derive(FuncMap)]
struct Test<'a, T>
where
    T: TestTrait,
{
    function_type: fn(T),
    macro_type: test_type!(),
    reference_type: &'a T,
    self_type: <T as TestTrait>::Assoc,
    slice_type: Box<[T]>,
    trait_object_type: Box<dyn Fn(T)>,
}

fn main() {}
