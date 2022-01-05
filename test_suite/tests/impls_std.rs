extern crate std;

use funcmap::FuncMap;

use std::{
    collections::{hash_map, hash_set, HashMap, HashSet},
    hash::Hash,
};

#[test]
fn field_of_hash_map_type_is_mapped_over_key() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(HashMap<T, ()>)
    where
        T: Eq + Hash;

    let src = Test([(T1, ())].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test([(T2, ())].into()));
}

#[test]
fn field_of_hash_map_type_is_mapped_over_value() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(HashMap<(), T>);

    let src = Test([((), T1)].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test([((), T2)].into()));
}

#[test]
fn field_of_hash_map_into_iter_type_is_mapped_over_key() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(hash_map::IntoIter<T, ()>);

    let src = Test(HashMap::from([(T1, ())]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [(T2, ())]);
}

#[test]
fn field_of_hash_map_into_iter_type_is_mapped_over_value() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(hash_map::IntoIter<(), T>);

    let src = Test(HashMap::from([((), T1)]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [((), T2)]);
}

#[test]
fn field_of_hash_set_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(HashSet<T>)
    where
        T: Eq + Hash;

    let src = Test([T1].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test([T2].into()));
}

#[test]
fn field_of_hash_set_into_iter_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(hash_set::IntoIter<T>);

    let src = Test(HashSet::from([T1]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [T2]);
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct T1;

#[derive(Debug, Eq, Hash, PartialEq)]
struct T2;
