use std::marker::PhantomData;

use funcmap::{FuncMap, TypeParam};

#[test]
fn struct_with_multiple_generics_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<S, T>(S, i32, T);

    let src = Test(T1, 42, T1);
    let dst = src.func_map_over::<TypeParam<0>, _>(|_| T2);
    assert_eq!(dst, Test(T2, 42, T1));

    let src = Test(T1, 42, T1);
    let dst = src.func_map_over::<TypeParam<1>, _>(|_| T2);
    assert_eq!(dst, Test(T1, 42, T2));
}

#[test]
fn struct_with_non_type_generics_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<'a, T, const N: usize>(T, PhantomData<&'a ()>);

    let src = Test::<'_, _, 42>(T1, PhantomData);
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test::<'_, _, 42>(T2, PhantomData));
}

#[test]
fn struct_with_const_generics_before_type_generics_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<const N: usize, S, const M: usize, T>(S, T);

    let src = Test::<42, _, 42, _>(T1, T1);
    let dst = src.func_map_over::<TypeParam<3>, _>(|_| T2);

    assert_eq!(dst, Test(T1, T2));
}

#[test]
fn field_of_generic_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Inner<'a, S, T, const N: usize>(S, T, PhantomData<&'a ()>);

    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<'a, S, T, const N: usize>(Inner<'a, S, T, N>);

    let src = Test::<'_, _, _, 42>(Inner(T1, T1, PhantomData));
    let dst = src.func_map_over::<TypeParam<0>, _>(|_| T2);
    assert_eq!(dst, Test::<'_, _, _, 42>(Inner(T2, T1, PhantomData)));

    let src = Test::<'_, _, _, 42>(Inner(T1, T1, PhantomData));
    let dst = src.func_map_over::<TypeParam<1>, _>(|_| T2);
    assert_eq!(dst, Test::<'_, _, _, 42>(Inner(T1, T2, PhantomData)));
}

#[test]
fn field_of_repeated_generic_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Inner<'a, S, T, const N: usize>(S, T, PhantomData<&'a ()>);

    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<'a, T, const N: usize>(Inner<'a, T, T, N>);

    let src = Test::<'_, _, 42>(Inner(T1, T1, PhantomData));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test::<'_, _, 42>(Inner(T2, T2, PhantomData)));
}

#[test]
fn field_of_generic_type_with_const_literal_before_generic_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Inner<const N: usize, S, const M: usize, T>(S, T);

    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(Inner<42, T, 42, T>);

    let src = Test(Inner(T1, T1));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(Inner(T2, T2)));
}

#[test]
fn field_of_generic_type_with_const_alias_before_generic_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Inner<const N: usize, S, const M: usize, T>(S, T);

    const N: usize = 42;

    // here the derive macro cannot know whether `N` is a type or a const
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(Inner<N, T, N, T>);

    let src = Test(Inner(T1, T1));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(Inner(T2, T2)));
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
