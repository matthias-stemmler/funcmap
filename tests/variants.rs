use mapstruct::MapStruct;

#[test]
fn tuple_struct_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<T>(T, i32, T);

    let src = Test(T1, 42, T1);
    let dst = src.map_struct(|_| T2);

    assert_eq!(dst, Test(T2, 42, T2));
}

#[test]
fn struct_with_named_fields_is_mapped() {
    #[derive(MapStruct, Debug, PartialEq)]
    struct Test<T> {
        value0: T,
        value1: i32,
        value2: T,
    }

    let src = Test {
        value0: T1,
        value1: 42,
        value2: T1,
    };
    let dst = src.map_struct(|_| T2);

    assert_eq!(
        dst,
        Test {
            value0: T2,
            value1: 42,
            value2: T2
        }
    );
}

// TODO enum case

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
