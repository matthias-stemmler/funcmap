use funcmap::{FuncMap, TypeParam};

#[test]
fn generics_to_be_mapped_can_be_configured() {
    fn noop() {}

    #[derive(FuncMap, Debug, PartialEq)]
    #[funcmap(params(S, T))]
    struct Test<S, T, U> {
        value1: S,
        value2: T,
        not_mappable: fn() -> U,
    }

    let src = Test {
        value1: T1,
        value2: T1,
        not_mappable: noop,
    };
    let dst = src
        .func_map_over::<TypeParam<0>, _>(|_| T2)
        .func_map_over::<TypeParam<1>, _>(|_| T2);

    assert_eq!(
        dst,
        Test {
            value1: T2,
            value2: T2,
            not_mappable: noop
        }
    );
}

#[test]
fn opts_accept_trailing_comma() {
    #[derive(FuncMap)]
    #[funcmap(params(S), params(T))]
    struct Test<S, T>(S, T);
}

#[test]
fn params_opt_accepts_trailing_comma() {
    #[derive(FuncMap)]
    #[funcmap(params(S, T,))]
    struct Test<S, T>(S, T);
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
