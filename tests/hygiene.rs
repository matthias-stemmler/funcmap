use funcmap::FuncMap;

#[test]
fn conflicting_idents_are_avoided() {
    #[allow(non_snake_case)]
    #[derive(FuncMap)]
    struct A<B, F, const C: usize> {
        B: B,
        F: F,
    }
}

#[test]
fn nested_items_are_not_mistaken_for_generics() {
    mod test {
        pub struct T;
    }

    #[derive(FuncMap)]
    struct Test<T>(T, test::T);
}
