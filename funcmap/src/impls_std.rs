extern crate std;

use crate::{FuncMap, TypeParam};

use core::hash::Hash;
use std::{
    collections::{hash_map, hash_set, HashMap, HashSet},
    io::{BufReader, Chain, Cursor, Read, Take},
};

impl<A, B> FuncMap<A, B> for BufReader<A>
where
    B: Read,
{
    type Output = BufReader<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        BufReader::new(f(self.into_inner()))
    }
}

impl<A, B, U> FuncMap<A, B, TypeParam<0>> for Chain<A, U>
where
    B: Read,
    U: Read,
{
    type Output = Chain<B, U>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        let (first, second) = self.into_inner();
        f(first).chain(second)
    }
}

impl<T, A, B> FuncMap<A, B, TypeParam<1>> for Chain<T, A>
where
    T: Read,
    B: Read,
{
    type Output = Chain<T, B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        let (first, second) = self.into_inner();
        first.chain(f(second))
    }
}

impl<A, B> FuncMap<A, B> for Cursor<A> {
    type Output = Cursor<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        Cursor::new(f(self.into_inner()))
    }
}

impl<A, B, V> FuncMap<A, B, TypeParam<0>> for HashMap<A, V>
where
    B: Eq + Hash,
{
    type Output = HashMap<B, V>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(|(k, v)| (f(k), v)).collect()
    }
}

impl<K, A, B> FuncMap<A, B, TypeParam<1>> for HashMap<K, A>
where
    K: Eq + Hash,
{
    type Output = HashMap<K, B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(|(k, v)| (k, f(v))).collect()
    }
}

impl<A, B, V> FuncMap<A, B, TypeParam<0>> for hash_map::IntoIter<A, V>
where
    B: Eq + Hash,
{
    type Output = hash_map::IntoIter<B, V>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(|(k, v)| (f(k), v))
            .collect::<HashMap<_, _>>()
            .into_iter()
    }
}

impl<K, A, B> FuncMap<A, B, TypeParam<1>> for hash_map::IntoIter<K, A>
where
    K: Eq + Hash,
{
    type Output = hash_map::IntoIter<K, B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(|(k, v)| (k, f(v)))
            .collect::<HashMap<_, _>>()
            .into_iter()
    }
}

impl<A, B> FuncMap<A, B> for HashSet<A>
where
    B: Eq + Hash,
{
    type Output = HashSet<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for hash_set::IntoIter<A>
where
    B: Eq + Hash,
{
    type Output = hash_set::IntoIter<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f).collect::<HashSet<_>>().into_iter()
    }
}

impl<A, B> FuncMap<A, B> for Take<A>
where
    B: Read,
{
    type Output = Take<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        let limit = self.limit();
        f(self.into_inner()).take(limit)
    }
}
