//! Implementations of [`FuncMap`](crate::FuncMap) and
//! [`TryFuncMap`](crate::TryFuncMap) for types in [`std`]

/// Implementations for types in [`std::collections::hash_map`]
mod hash_map {
    use crate::{FuncMap, TryFuncMap, TypeParam};

    use core::hash::Hash;
    use std::collections::{hash_map, HashMap};

    impl<A, B, V, S> FuncMap<A, B, TypeParam<0>> for HashMap<A, V, S>
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

    impl<A, B, V, S> TryFuncMap<A, B, TypeParam<0>> for HashMap<A, V, S>
    where
        B: Eq + Hash,
    {
        type Output = HashMap<B, V>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.into_iter().map(|(k, v)| Ok((f(k)?, v))).collect()
        }
    }

    impl<K, A, B, S> FuncMap<A, B, TypeParam<1>> for HashMap<K, A, S>
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

    impl<K, A, B, S> TryFuncMap<A, B, TypeParam<1>> for HashMap<K, A, S>
    where
        K: Eq + Hash,
    {
        type Output = HashMap<K, B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
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

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(|(k, v)| (f(k), v))
                .collect::<HashMap<_, _>>()
                .into_iter()
        }
    }

    impl<A, B, V> TryFuncMap<A, B, TypeParam<0>> for hash_map::IntoIter<A, V>
    where
        B: Eq + Hash,
    {
        type Output = hash_map::IntoIter<B, V>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
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

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(|(k, v)| (k, f(v)))
                .collect::<HashMap<_, _>>()
                .into_iter()
        }
    }

    impl<K, A, B> TryFuncMap<A, B, TypeParam<1>> for hash_map::IntoIter<K, A>
    where
        K: Eq + Hash,
    {
        type Output = hash_map::IntoIter<K, B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(|(k, v)| Ok((k, f(v)?)))
                .collect::<Result<HashMap<_, _>, _>>()
                .map(IntoIterator::into_iter)
        }
    }
}

/// Implementations for types in [`std::collections::hash_set`]
mod hash_set {
    use crate::{FuncMap, TryFuncMap};

    use core::hash::Hash;
    use std::collections::{hash_set, HashSet};

    impl<A, B, S> FuncMap<A, B> for HashSet<A, S>
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

    impl<A, B, S> TryFuncMap<A, B> for HashSet<A, S>
    where
        B: Eq + Hash,
    {
        type Output = HashSet<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
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

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(f).collect::<HashSet<_>>().into_iter()
        }
    }

    impl<A, B> TryFuncMap<A, B> for hash_set::IntoIter<A>
    where
        B: Eq + Hash,
    {
        type Output = hash_set::IntoIter<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(f)
                .collect::<Result<HashSet<_>, _>>()
                .map(IntoIterator::into_iter)
        }
    }
}
