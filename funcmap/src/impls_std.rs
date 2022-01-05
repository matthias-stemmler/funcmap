extern crate std;

use crate::{FuncMap, TypeParam};

use core::hash::Hash;
use std::collections::{hash_map, hash_set, HashMap, HashSet};

impl<A, B, V> FuncMap<A, B, TypeParam<0>> for HashMap<A, V>
where
    B: Eq + Hash,
{
    type Output = HashMap<B, V>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(|(k, v)| Ok((f(k)?, v))).collect()
    }
}

impl<K, A, B> FuncMap<A, B, TypeParam<1>> for HashMap<K, A>
where
    K: Eq + Hash,
{
    type Output = HashMap<K, B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(|(k, v)| Ok((k, f(v)?))).collect()
    }
}

impl<A, B, V> FuncMap<A, B, TypeParam<0>> for hash_map::IntoIter<A, V>
where
    B: Eq + Hash,
{
    type Output = hash_map::IntoIter<B, V>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(|(k, v)| Ok((f(k)?, v)))
            .collect::<Result<HashMap<_, _>, _>>()
            .map(IntoIterator::into_iter)
    }
}

impl<K, A, B> FuncMap<A, B, TypeParam<1>> for hash_map::IntoIter<K, A>
where
    K: Eq + Hash,
{
    type Output = hash_map::IntoIter<K, B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(|(k, v)| Ok((k, f(v)?)))
            .collect::<Result<HashMap<_, _>, _>>()
            .map(IntoIterator::into_iter)
    }
}

impl<A, B> FuncMap<A, B> for HashSet<A>
where
    B: Eq + Hash,
{
    type Output = HashSet<B>;

    fn try_func_map<F, E>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.into_iter().map(f).collect()
    }
}

impl<A, B> FuncMap<A, B> for hash_set::IntoIter<A>
where
    B: Eq + Hash,
{
    type Output = hash_set::IntoIter<B>;

    fn try_func_map<F, E>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(f)
            .collect::<Result<HashSet<_>, _>>()
            .map(IntoIterator::into_iter)
    }
}
