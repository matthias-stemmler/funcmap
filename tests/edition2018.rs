use funcmap::FuncMap;

#[test]
fn bare_trait_objects_are_supported() {
    #[derive(FuncMap, Debug, PartialEq)]
    #[allow(bare_trait_objects)]
    struct Test<T>(T)
    where
        Box<Send>: Send; // from edition 2021 onwards, using `Box<Send>` rather than `Box<dyn Send>` is a hard error

    let src = Test(T1);
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(T2));
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
