use funcmap::TryFuncMap;

use std::convert::{TryFrom, TryInto};

#[test]
fn mapping_succeeds_when_function_succeeds() {
    #[derive(TryFuncMap, Debug, PartialEq)]
    struct Test<T>(T, T, T);

    let src = Test(T1::Mappable, T1::Mappable, T1::Mappable);
    let dst: Result<Test<T2>, _> = src.try_func_map(TryInto::try_into);

    assert_eq!(dst, Ok(Test(T2, T2, T2)));
}

#[test]
fn mapping_fails_with_first_error_when_function_fails() {
    #[derive(TryFuncMap, Debug, PartialEq)]
    struct Test<T>(T, T, T);

    let src = Test(
        T1::NotMappable("First Error"),
        T1::Mappable,
        T1::NotMappable("Second Error"),
    );
    let dst: Result<Test<T2>, _> = src.try_func_map(TryInto::try_into);

    assert_eq!(dst, Err(MappingError("First Error")));
}

#[derive(Debug, PartialEq)]
enum T1 {
    Mappable,
    NotMappable(&'static str),
}

#[derive(Debug, PartialEq)]
struct T2;

impl TryFrom<T1> for T2 {
    type Error = MappingError;

    fn try_from(value: T1) -> Result<Self, Self::Error> {
        match value {
            T1::Mappable => Ok(T2),
            T1::NotMappable(message) => Err(MappingError(message)),
        }
    }
}

#[derive(Debug, PartialEq)]
struct MappingError(&'static str);
