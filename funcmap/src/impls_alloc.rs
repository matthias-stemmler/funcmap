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

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for binary_heap::IntoIter<A>
where
    B: Ord,
{
    type Output = binary_heap::IntoIter<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(f)
            .collect::<Result<BinaryHeap<_>, _>>()
            .map(IntoIterator::into_iter)
    }
}

impl<A, B> FuncMap<A, B> for Box<A> {
    type Output = Box<B>;

    fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(f(*self)?.into())
    }
}

impl<A, B, V> FuncMap<A, B, TypeParam<0>> for BTreeMap<A, V>
where
    B: Ord,
{
    type Output = BTreeMap<B, V>;

    fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(|(k, v)| Ok((f(k)?, v))).collect()
    }
}

impl<K, A, B> FuncMap<A, B, TypeParam<1>> for BTreeMap<K, A>
where
    K: Ord,
{
    type Output = BTreeMap<K, B>;

    fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(|(k, v)| Ok((k, f(v)?))).collect()
    }
}

impl<A, B, V> FuncMap<A, B, TypeParam<0>> for btree_map::IntoIter<A, V>
where
    B: Ord,
{
    type Output = btree_map::IntoIter<B, V>;

    fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(|(k, v)| Ok((f(k)?, v)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(IntoIterator::into_iter)
    }
}

impl<K, A, B> FuncMap<A, B, TypeParam<1>> for btree_map::IntoIter<K, A>
where
    K: Ord,
{
    type Output = btree_map::IntoIter<K, B>;

    fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(|(k, v)| Ok((k, f(v)?)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(IntoIterator::into_iter)
    }
}

impl<A, B> FuncMap<A, B> for BTreeSet<A>
where
    B: Ord,
{
    type Output = BTreeSet<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for btree_set::IntoIter<A>
where
    B: Ord,
{
    type Output = btree_set::IntoIter<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(f)
            .collect::<Result<BTreeSet<_>, _>>()
            .map(IntoIterator::into_iter)
    }
}

impl<A, B> FuncMap<A, B> for LinkedList<A> {
    type Output = LinkedList<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for linked_list::IntoIter<A> {
    type Output = linked_list::IntoIter<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(f)
            .collect::<Result<LinkedList<_>, _>>()
            .map(IntoIterator::into_iter)
    }
}

impl<A, B> FuncMap<A, B> for Vec<A> {
    type Output = Vec<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for vec::IntoIter<A> {
    type Output = vec::IntoIter<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(f)
            .collect::<Result<Vec<_>, _>>()
            .map(IntoIterator::into_iter)
    }
}

impl<A, B> FuncMap<A, B> for VecDeque<A> {
    type Output = VecDeque<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for vec_deque::IntoIter<A> {
    type Output = vec_deque::IntoIter<B>;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(f)
            .collect::<Result<VecDeque<_>, _>>()
            .map(IntoIterator::into_iter)
    }
}
