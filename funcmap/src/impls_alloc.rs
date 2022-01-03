extern crate alloc;

use crate::{FuncMap, TypeParam};

use alloc::boxed::Box;
use alloc::collections::{
    binary_heap, btree_map, btree_set, linked_list, vec_deque, BTreeMap, BTreeSet, BinaryHeap,
    LinkedList, VecDeque,
};
use alloc::vec::{self, Vec};

impl<A, B> FuncMap<A, B> for BinaryHeap<A>
where
    B: Ord,
{
    type Output = BinaryHeap<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for binary_heap::IntoIter<A>
where
    B: Ord,
{
    type Output = binary_heap::IntoIter<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f).collect::<BinaryHeap<_>>().into_iter()
    }
}

impl<A, B> FuncMap<A, B> for Box<A> {
    type Output = Box<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        Box::new(f(*self))
    }
}

impl<A, B, V> FuncMap<A, B, TypeParam<0>> for BTreeMap<A, V>
where
    B: Ord,
{
    type Output = BTreeMap<B, V>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(|(k, v)| (f(k), v)).collect()
    }
}

impl<K, A, B> FuncMap<A, B, TypeParam<1>> for BTreeMap<K, A>
where
    K: Ord,
{
    type Output = BTreeMap<K, B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(|(k, v)| (k, f(v))).collect()
    }
}

impl<A, B, V> FuncMap<A, B, TypeParam<0>> for btree_map::IntoIter<A, V>
where
    B: Ord,
{
    type Output = btree_map::IntoIter<B, V>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(|(k, v)| (f(k), v))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
    }
}

impl<K, A, B> FuncMap<A, B, TypeParam<1>> for btree_map::IntoIter<K, A>
where
    K: Ord,
{
    type Output = btree_map::IntoIter<K, B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(|(k, v)| (k, f(v)))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
    }
}

impl<A, B> FuncMap<A, B> for BTreeSet<A>
where
    B: Ord,
{
    type Output = BTreeSet<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for btree_set::IntoIter<A>
where
    B: Ord,
{
    type Output = btree_set::IntoIter<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f).collect::<BTreeSet<_>>().into_iter()
    }
}

impl<A, B> FuncMap<A, B> for LinkedList<A> {
    type Output = LinkedList<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for linked_list::IntoIter<A> {
    type Output = linked_list::IntoIter<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f).collect::<LinkedList<_>>().into_iter()
    }
}

impl<A, B> FuncMap<A, B> for Vec<A> {
    type Output = Vec<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for vec::IntoIter<A> {
    type Output = vec::IntoIter<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f).collect::<Vec<_>>().into_iter()
    }
}

impl<A, B> FuncMap<A, B> for VecDeque<A> {
    type Output = VecDeque<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for vec_deque::IntoIter<A> {
    type Output = vec_deque::IntoIter<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f).collect::<VecDeque<_>>().into_iter()
    }
}
