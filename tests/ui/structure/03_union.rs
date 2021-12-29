use core::marker::PhantomData;
use funcmap::FuncMap;

#[derive(FuncMap)]
union Test<T> {
    value: PhantomData<T>,
}

fn main() {}
