extern crate funcmap;

use funcmap::FuncMap;
use std::fmt::Debug;

#[test]
fn edition_2018_keywords_are_supported_as_identifiers() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T> {
        // from edition 2018 onwards, these field names are keywords
        async: T,
        await: T,
        try: T,
    }

    let src = Test {
        async: T1,
        await: T1,
        try: T1,
    };
    let dst = src.func_map(|_| T2);

    assert_eq!(
        dst,
        Test {
            async: T2,
            await: T2,
            try: T2,
        }
    );
}

#[test]
fn leading_colon_referring_to_crate_root_is_supported() {
    #[derive(FuncMap, Debug, PartialEq)]
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
