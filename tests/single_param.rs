use funcmap::FuncMap;

#[test]
fn field_of_generic_param_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(T);

    let src = Test(T1);
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(T2));
}

#[test]
fn field_of_generic_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Inner<T>(T);

    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(Inner<T>);

    let src = Test(Inner(T1));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(Inner(T2)));
}

#[test]
fn field_of_nested_generic_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Inner<T>(T);

    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(Inner<Inner<T>>);

    let src = Test(Inner(Inner(T1)));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(Inner(Inner(T2))));
}

#[test]
fn field_of_non_generic_type_is_not_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(T, i32);

    let src = Test(T1, 42);
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(T2, 42));
}

#[test]
fn tuple_entry_of_generic_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>((T, i32, T));

    let src = Test((T1, 42, T1));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test((T2, 42, T2)));
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
