use std::marker::PhantomData;

use mapstruct::{MapStruct, TypeParam};

#[test]
fn struct_with_multiple_generics_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S, T>(S, i32, T);

    let src = Test(T1, 42, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);
    assert_eq!(dst, Test(T2, 42, T1));

    let src = Test(T1, 42, T1);
    let dst = src.map_struct_over(TypeParam::<1>, |_| T2);
    assert_eq!(dst, Test(T1, 42, T2));
}

#[test]
fn struct_with_non_type_generics_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<'a, T, const N: usize>(T, PhantomData<&'a ()>);

    let src = Test::<'_, _, 42>(T1, PhantomData);
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test::<'_, _, 42>(T2, PhantomData));
}

#[test]
fn field_of_generic_type_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Inner<'a, S, T, const N: usize>(S, T, PhantomData<&'a ()>);

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<'a, S, T, const N: usize>(Inner<'a, S, T, N>);

    let src = Test::<'_, _, _, 42>(Inner(T1, T1, PhantomData));
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);
    assert_eq!(
        dst,
        Test::<'_, _, _, 42>(Inner(T2, T1, PhantomData))
    );

    let src = Test::<'_, _, _, 42>(Inner(T1, T1, PhantomData));
    let dst = src.map_struct_over(TypeParam::<1>, |_| T2);
    assert_eq!(
        dst,
        Test::<'_, _, _, 42>(Inner(T1, T2, PhantomData))
    );
}

#[test]
fn field_of_repeated_generic_type_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Inner<'a, S, T, const N: usize>(S, T, PhantomData<&'a ()>);

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<'a, T, const N: usize>(Inner<'a, T, T, N>);

    let src = Test::<'_, _, 42>(Inner(T1, T1, PhantomData));
    let dst = src.map_struct(|_| T2);

    assert_eq!(
        dst,
        Test::<'_, _, 42>(Inner(T2, T2, PhantomData))
    );
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
