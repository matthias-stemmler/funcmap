use core::marker::PhantomData;
use mapstruct::MapStruct;

#[test]
fn field_of_array_type_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<T>([T; 2]);

    let src = Test([T1, T1]);
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test([T2, T2]));
}

#[test]
fn field_of_option_type_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<T>(Option<T>);

    let src = Test(Some(T1));
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test(Some(T2)));
}

#[test]
fn field_of_result_type_is_mapped_over_value() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<T>(Result<T, ()>);

    let src = Test(Ok(T1));
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test(Ok(T2)));
}

#[test]
fn field_of_result_type_is_mapped_over_error() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<T>(Result<(), T>);

    let src = Test(Err(T1));
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test(Err(T2)));
}

#[test]
fn field_of_phantom_data_type_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<T>(PhantomData<T>);

    let src = Test(PhantomData::<T1>);
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test(PhantomData::<T2>));
}

#[cfg(feature = "alloc")]
mod alloc {
    use super::*;

    #[test]
    fn field_of_box_type_is_mapped() {
        #[derive(MapStruct, Debug, PartialEq)]
        struct Test<T>(Box<T>);

        let src = Test(Box::new(T1));
        let dst = src.map_struct(|_| T2);

        assert_eq!(dst, Test(Box::new(T2)));
    }

    #[test]
    fn field_of_vec_type_is_mapped() {
        #[derive(MapStruct, Debug, PartialEq)]
        struct Test<T>(Vec<T>);

        let src = Test(vec![T1, T1]);
        let dst = src.map_struct(|_| T2);

        assert_eq!(dst, Test(vec![T2, T2]));
    }
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
