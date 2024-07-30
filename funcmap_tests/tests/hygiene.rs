#![allow(dead_code)]

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
fn inherent_methods_are_not_mistaken_for_trait_methods() {
    #[derive(FuncMap)]
    struct Inner<T>(T);

    #[allow(dead_code)]
    impl<T> Inner<T> {
        fn func_map(self) {}
        fn func_map_over(self) {}
    }

    #[derive(FuncMap)]
    struct Test<T>(Inner<T>);
}

#[test]
fn trait_methods_are_not_confused_with_methods_of_other_traits() {
    #[derive(FuncMap)]
    struct Inner<T>(T);

    #[allow(dead_code)]
    trait LikeFuncMap: Sized {
        fn func_map(self) {}
        fn func_map_over(self) {}
    }

    impl<T> LikeFuncMap for Inner<T> {}

    impl<T, const N: usize> LikeFuncMap for [T; N] {}

    #[derive(FuncMap)]
    struct Test<T, const N: usize>(Inner<T>, [T; N]);
}

#[test]
fn raw_identifiers_are_supported() {
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]

    use funcmap as r#break;

    #[derive(FuncMap)]
    struct r#continue<r#else>(r#else);

    #[derive(r#break::FuncMap)]
    #[funcmap(crate = "r#break", params(r#extern))]
    enum r#false<r#extern, const r#for: usize> {
        r#if { r#in: r#continue<r#extern> },
        r#let,
    }
}
