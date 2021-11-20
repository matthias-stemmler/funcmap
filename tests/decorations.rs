use mapstruct::{MapStruct, TypeParam};

#[test]
fn attributes_on_generics_are_supported() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<#[cfg(test)] S, #[cfg(test)] T>(S, T);

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn defaults_on_generics_are_supported() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S, T = T1>(S, T);

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn impl_is_restricted_to_trait_bounds_on_generics_of_original_type() {
    trait TestTrait {}

    impl TestTrait for T1 {}
    impl TestTrait for T2 {}

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S: TestTrait, T: TestTrait>(S, T);

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn impl_is_restricted_to_self_dependent_trait_bounds_on_generics_of_original_type() {
    trait TestTrait<T> {
        type Assoc;
    }

    impl<S> TestTrait<S> for T1 {
        type Assoc = S;
    }

    impl<S> TestTrait<S> for T2 {
        type Assoc = S;
    }

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S: TestTrait<S, Assoc = S>, T: TestTrait<T, Assoc = T>>(S, T);

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn impl_is_restricted_to_cross_dependent_trait_bounds_on_generics_of_original_type() {
    trait TestTrait<T> {
        type Assoc;
    }

    impl<S> TestTrait<S> for T1 {
        type Assoc = S;
    }

    impl<S> TestTrait<S> for T2 {
        type Assoc = S;
    }

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S: TestTrait<T, Assoc = T>, T: TestTrait<S, Assoc = S>>(S, T);

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn impl_is_restricted_to_trait_bounds_in_where_clause_on_original_type() {
    trait TestTrait {}

    impl TestTrait for T1 {}
    impl TestTrait for T2 {}

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S, T>(S, T)
    where
        S: TestTrait,
        T: TestTrait;

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn impl_is_restricted_to_self_dependent_trait_bounds_in_where_clause_on_original_type() {
    trait TestTrait<T> {
        type Assoc;
    }

    impl<S> TestTrait<S> for T1 {
        type Assoc = S;
    }

    impl<S> TestTrait<S> for T2 {
        type Assoc = S;
    }

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S, T>(S, T)
    where
        S: TestTrait<S, Assoc = S>,
        T: TestTrait<T, Assoc = T>;

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn impl_is_restricted_to_cross_dependent_trait_bounds_in_where_clause_on_original_type() {
    trait TestTrait<T> {
        type Assoc;
    }

    impl<S> TestTrait<S> for T1 {
        type Assoc = S;
    }

    impl<S> TestTrait<S> for T2 {
        type Assoc = S;
    }

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S, T>(S, T)
    where
        S: TestTrait<T, Assoc = T>,
        T: TestTrait<S, Assoc = S>;

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn impl_is_restricted_to_arbitrary_trait_bounds_in_where_clause_on_original_type() {
    trait TestTrait<T> {
        type Assoc;
    }

    impl<S> TestTrait<S> for T1 {
        type Assoc = S;
    }

    impl<S> TestTrait<S> for T2 {
        type Assoc = S;
    }

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S, T>(S, T)
    where
        S: TestTrait<T>,
        <S as TestTrait<T>>::Assoc: TestTrait<S>,
        <<S as TestTrait<T>>::Assoc as TestTrait<S>>::Assoc: TestTrait<T>;

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
}

#[test]
fn impl_is_restricted_to_trait_bounds_with_bound_lifetimes_in_where_clause_on_original_type() {
    trait TestTrait<'a> {}

    impl<'a> TestTrait<'a> for T1 {}
    impl<'a> TestTrait<'a> for T2 {}

    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<S, T>(S, T)
    where
        for<'a> T: TestTrait<'a>;

    let src = Test(T1, T1);
    let dst = src.map_struct_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst, Test(T2, T1));
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