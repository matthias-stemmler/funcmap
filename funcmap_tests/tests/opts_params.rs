#![allow(dead_code)]

use funcmap::{FuncMap, TypeParam};

#[test]
fn generics_to_be_mapped_can_be_configured() {
    #[derive(FuncMap, Debug, PartialEq)]
    #[funcmap(params(S, U))]
    struct Test<S, T, U> {
        value1: S,
        not_mappable: *const T,
        value2: U,
    }

    let src = Test {
        value1: T1,
        not_mappable: std::ptr::null::<()>(),
        value2: T1,
    };
    let dst = src
        .func_map_over::<TypeParam<0>, _>(|_| T2)
        .func_map_over::<TypeParam<2>, _>(|_| T2);

    assert_eq!(
        dst,
        Test {
            value1: T2,
            not_mappable: std::ptr::null::<()>(),
            value2: T2,
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
