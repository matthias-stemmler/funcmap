use std::marker::PhantomData;

use mapstruct::{MapStruct, TypeParam};

#[test]
fn generic_param_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<T> {
        value: T,
    }

    let src = TestStruct { value: TestTypeA };
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(dst, TestStruct { value: TestTypeB });
}

#[test]
fn field_independent_of_generic_param_does_not_get_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<T> {
        dummy: T,
        value: i32,
    }

    let src = TestStruct {
        dummy: TestTypeA,
        value: 42,
    };
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(dst.value, 42);
}

#[test]
fn tuple_of_generic_param_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<T> {
        value: (T, i32, T),
    }

    let src = TestStruct {
        value: (TestTypeA, 42, TestTypeA),
    };
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(
        dst,
        TestStruct {
            value: (TestTypeB, 42, TestTypeB)
        }
    );
}

#[test]
fn array_of_generic_param_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<T> {
        value: [T; 2],
    }

    let src = TestStruct {
        value: [TestTypeA, TestTypeA],
    };
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(
        dst,
        TestStruct {
            value: [TestTypeB, TestTypeB]
        }
    );
}

#[test]
fn option_of_generic_param_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<T> {
        value: Option<T>,
    }

    let src = TestStruct {
        value: Some(TestTypeA),
    };
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(
        dst,
        TestStruct {
            value: Some(TestTypeB)
        }
    );
}

#[test]
fn phantom_data_of_generic_param_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<T> {
        value: PhantomData<T>,
    }

    let src = TestStruct::<TestTypeA> { value: PhantomData };
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(dst, TestStruct { value: PhantomData });
}

#[test]
fn generics_are_bounded_to_enable_mapping() {
    #[derive(Debug, PartialEq)]
    struct InnerTestStruct<T>(T);

    impl MapStruct<TestTypeA, TestTypeB> for InnerTestStruct<TestTypeA> {
        type Output = InnerTestStruct<TestTypeB>;

        fn map_struct<F>(self, _: F) -> Self::Output
        where
            F: FnMut(TestTypeA) -> TestTypeB,
        {
            InnerTestStruct(TestTypeB)
        }
    }

    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<T> {
        value: InnerTestStruct<T>,
    }

    let src = TestStruct {
        value: InnerTestStruct(TestTypeA),
    };
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(
        dst,
        TestStruct {
            value: InnerTestStruct(TestTypeB)
        }
    );
}

#[test]
fn tuple_struct_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<T>(i32, T, Option<T>);

    let src = TestStruct(42, TestTypeA, Some(TestTypeA));
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(dst, TestStruct(42, TestTypeB, Some(TestTypeB)));
}

#[test]
fn struct_with_multiple_generics_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<S, T>(S, T);

    let src = TestStruct(TestTypeA, TestTypeA);
    let dst = src.map_struct_over(TypeParam::<0>, |_| TestTypeB);
    assert_eq!(dst, TestStruct(TestTypeB, TestTypeA));

    let src = TestStruct(TestTypeA, TestTypeA);
    let dst = src.map_struct_over(TypeParam::<1>, |_| TestTypeB);
    assert_eq!(dst, TestStruct(TestTypeA, TestTypeB));
}

#[test]
fn struct_with_non_type_generics_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<'a, T, const N: usize> {
        value: T,
        phantom: PhantomData<&'a ()>,
    }

    let src = TestStruct::<'_, _, 0> {
        value: TestTypeA,
        phantom: PhantomData,
    };
    let dst = src.map_struct(|_| TestTypeB);

    assert_eq!(dst.value, TestTypeB);
}

#[test]
fn struct_with_nested_non_type_generics_gets_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct InnerTestStruct<'a, S, T, const N: usize> {
        value: (S, T),
        phantom: PhantomData<&'a ()>,
    }

    // TODO: use <'a, T, T, N> instead
    #[derive(MapStruct, Debug, PartialEq)]
    struct TestStruct<'a, T, const N: usize> {
        value: InnerTestStruct<'a, T, u8, N>,
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use super::*;

    #[test]
    fn vec_of_generic_param_gets_mapped() {
        #[derive(MapStruct, Debug, PartialEq)]
        struct TestStruct<T> {
            value: Vec<T>,
        }

        let src = TestStruct {
            value: vec![TestTypeA, TestTypeA],
        };
        let dst = src.map_struct(|_| TestTypeB);

        assert_eq!(
            dst,
            TestStruct {
                value: vec![TestTypeB, TestTypeB]
            }
        );
    }

    #[test]
    fn nested_structure_of_generic_param_gets_mapped() {
        #[derive(MapStruct, Debug, PartialEq)]
        struct TestStruct<T> {
            value: Vec<Option<Vec<T>>>,
        }

        let src = TestStruct {
            value: vec![Some(vec![TestTypeA, TestTypeA]), None, Some(vec![])],
        };
        let dst = src.map_struct(|_| TestTypeB);

        assert_eq!(
            dst,
            TestStruct {
                value: vec![Some(vec![TestTypeB, TestTypeB]), None, Some(vec![])]
            }
        );
    }
}

#[derive(Debug, PartialEq)]
struct TestTypeA;

#[derive(Debug, PartialEq)]
struct TestTypeB;
