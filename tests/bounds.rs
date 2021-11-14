use mapstruct::MapStruct;

#[test]
fn impl_is_restricted_to_bounds_on_original_type() {
    trait TestTrait {}

    impl TestTrait for T1 {}
    impl TestTrait for T2 {}

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<#[cfg(test)] T: TestTrait = T1>(T);

    let src = Test(T1);
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test(T2));
}

#[test]
fn impl_is_restricted_to_allow_mapping_of_inner_type() {
    #[derive(Debug, PartialEq)]
    struct Inner<T>(T);

    // impl manually only for Inner<T1>, not generic Inner<T>
    impl MapStruct<T1, T2> for Inner<T1> {
        type Output = Inner<T2>;

        fn map_struct<F>(self, _: F) -> Self::Output
        where
            F: FnMut(T1) -> T2,
        {
            Inner(T2)
        }
    }

    // derived impl is supposed to have a clause
    // `where Inner<A>: MapStruct<A, B, Output = Inner<B>>`
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<T>(Inner<T>);

    let src = Test(Inner(T1));
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test(Inner(T2)));
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
