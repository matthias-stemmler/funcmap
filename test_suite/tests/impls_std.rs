extern crate std;

use funcmap::{FuncMap, TypeParam};

use std::{
    collections::{hash_map, hash_set, HashMap, HashSet},
    hash::Hash,
    io::{self, BufReader, Chain, Cursor, Read, Take},
};

#[test]
fn field_of_buf_reader_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<R>(BufReader<R>);

    let src = Test(BufReader::new(T1));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.into_inner(), T2);
}

#[test]
fn field_of_chain_type_is_mapped_over_first() {
    #[derive(FuncMap, Debug)]
    struct Test<T, U>(Chain<T, U>);

    let src = Test(T1.chain(T1));
    let dst = src.func_map_over(TypeParam::<0>, |_| T2);

    assert_eq!(dst.0.into_inner().0, T2);
}

#[test]
fn field_of_chain_type_is_mapped_over_second() {
    #[derive(FuncMap, Debug)]
    struct Test<T, U>(Chain<T, U>);

    let src = Test(T1.chain(T1));
    let dst = src.func_map_over(TypeParam::<1>, |_| T2);

    assert_eq!(dst.0.into_inner().1, T2);
}

#[test]
fn field_of_cursor_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(Cursor<T>);

    let src = Test(Cursor::new(T1));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(Cursor::new(T2)));
}

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

#[test]
fn field_of_take_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(Take<T>);

    let src = Test(T1.take(42));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.limit(), 42);
    assert_eq!(dst.0.into_inner(), T2);
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct T1;

#[derive(Debug, Eq, Hash, PartialEq)]
struct T2;

impl Read for T1 {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }
}

impl Read for T2 {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }
}
