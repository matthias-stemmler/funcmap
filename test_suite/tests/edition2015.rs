extern crate funcmap;

use funcmap::FuncMap;
use std::fmt::Debug;

#[test]
fn edition_2018_keywords_are_supported_as_identifiers() {
    // deny this here to make sure it is explicitly allowed for the derived impl
    #![deny(keyword_idents)]

    #[derive(FuncMap, Debug, PartialEq)]
    #[allow(keyword_idents)]
    struct Test<T> {
        // from edition 2018 onwards, these field names are keywords
        async: T,
        await: T,
        try: T,
    }

    let src = Test {
        r#async: T1,
        r#await: T1,
        r#try: T1,
    };
    let dst = src.func_map(|_| T2);

    assert_eq!(
        dst,
        Test {
            r#async: T2,
            r#await: T2,
            r#try: T2,
        }
    );
}

#[test]
fn leading_colon_referring_to_crate_root_is_supported() {
    // deny this here to make sure it is explicitly allowed for the derived impl
    #![deny(absolute_paths_not_starting_with_crate)]

    #[derive(FuncMap, Debug, PartialEq)]
    #[allow(absolute_paths_not_starting_with_crate)]
    struct Test<T>(T)
    where
        ::T1: Debug; // from edition 2018 onwards, `::T1` must refer to an external crate `T1`

    let src = Test(T1);
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(T2));
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
