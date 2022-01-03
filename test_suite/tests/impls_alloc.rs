extern crate alloc;

use funcmap::FuncMap;

use alloc::{
    collections::{
        binary_heap, btree_map, btree_set, linked_list, vec_deque, BTreeMap, BTreeSet, BinaryHeap,
        LinkedList, VecDeque,
    },
    vec,
};

#[test]
fn field_of_binary_heap_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(BinaryHeap<T>);

    let src = Test([T1].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.into_vec(), [T2]);
}

#[test]
fn field_of_binary_heap_into_iter_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(binary_heap::IntoIter<T>);

    let src = Test(BinaryHeap::from([T1]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [T2]);
}

#[test]
fn field_of_box_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(Box<T>);

    let src = Test(Box::new(T1));
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(Box::new(T2)));
}

#[test]
fn field_of_btree_map_type_is_mapped_over_key() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(BTreeMap<T, ()>);

    let src = Test([(T1, ())].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test([(T2, ())].into()));
}

#[test]
fn field_of_btree_map_type_is_mapped_over_value() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(BTreeMap<(), T>);

    let src = Test([((), T1)].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test([((), T2)].into()));
}

#[test]
fn field_of_btree_map_into_iter_type_is_mapped_over_key() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(btree_map::IntoIter<T, ()>);

    let src = Test(BTreeMap::from([(T1, ())]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [(T2, ())]);
}

#[test]
fn field_of_btree_map_into_iter_type_is_mapped_over_value() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(btree_map::IntoIter<(), T>);

    let src = Test(BTreeMap::from([((), T1)]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [((), T2)]);
}

#[test]
fn field_of_btree_set_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(BTreeSet<T>);

    let src = Test([T1].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test([T2].into()));
}

#[test]
fn field_of_btree_set_into_iter_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(btree_set::IntoIter<T>);

    let src = Test(BTreeSet::from([T1]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [T2]);
}

#[test]
fn field_of_linked_list_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(LinkedList<T>);

    let src = Test([T1].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test([T2].into()));
}

#[test]
fn field_of_linked_list_into_iter_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(linked_list::IntoIter<T>);

    let src = Test(LinkedList::from([T1]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [T2]);
}

#[test]
fn field_of_vec_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(Vec<T>);

    let src = Test(vec![T1, T1]);
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test(vec![T2, T2]));
}

#[test]
fn field_of_vec_into_iter_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(vec::IntoIter<T>);

    let src = Test(vec![T1, T1].into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [T2, T2]);
}

#[test]
fn field_of_vec_deque_type_is_mapped() {
    #[derive(FuncMap, Debug, PartialEq)]
    struct Test<T>(VecDeque<T>);

    let src = Test([T1, T1].into());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst, Test([T2, T2].into()));
}

#[test]
fn field_of_vec_deque_into_iter_type_is_mapped() {
    #[derive(FuncMap, Debug)]
    struct Test<T>(vec_deque::IntoIter<T>);

    let src = Test(VecDeque::from([T1, T1]).into_iter());
    let dst = src.func_map(|_| T2);

    assert_eq!(dst.0.collect::<Vec<_>>(), [T2, T2]);
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct T1;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct T2;
