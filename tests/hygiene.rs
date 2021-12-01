use funcmap::FuncMap;

#[test]
fn conflicting_type_params_are_avoided() {
    #[allow(non_snake_case)]
    #[derive(FuncMap)]
    struct A<B, F, const C: usize> {
        D: (),
        B: B,
        F: F,
    }
}

#[test]
fn fields_conflicting_with_items_are_supported() {
    #[allow(non_snake_case)]
    #[derive(FuncMap)]
    struct Test<T> {
        funcmap: T,
        FuncMap: T,
        TypeParam: T,
        Output: T,
        func_map: T,
        f: T,
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

#[test]
fn lints_are_not_denied_in_emitted_code() {
    #![deny(non_camel_case_types)]

    #[allow(non_camel_case_types)]
    #[derive(FuncMap)]
    struct Test<s, t>(s, t);
}
