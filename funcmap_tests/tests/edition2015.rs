// This file uses edition = "2015" via Cargo.toml

extern crate funcmap;

use funcmap::FuncMap;
use std::fmt::Debug;

#[test]
fn edition_2018_keywords_are_supported_as_raw_identifiers() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T> {
        // from edition 2018 onwards, these field names are keywords
        // we still require them to be used as raw identifiers in order to be able to parse them with syn 2.x
        r#async: T,
        r#await: T,
        r#dyn: T,
        r#try: T,
    }

    let src = Test {
        async: T1,
        await: T1,
        dyn: T1,
        try: T1,
    };
    let dst = src.func_map(|_| T2);

    assert_eq!(
        dst,
        Test {
            async: T2,
            await: T2,
            dyn: T2,
            try: T2,
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
