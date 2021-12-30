extern crate alloc;

use crate::{FuncMap, TypeParam};

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap};
use alloc::vec::Vec;

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

impl<A, B> FuncMap<A, B> for Vec<A> {
    type Output = Vec<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}
