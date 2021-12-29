#![no_std]
#![deny(missing_debug_implementations)]

use core::cell::Cell;
use core::marker::PhantomData;

#[doc(hidden)]
pub use funcmap_derive::*;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeParam<const N: usize>;

pub trait FuncMap<A, B, P = TypeParam<0>>: Sized {
    type Output;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B;

    fn func_map_over<F>(self, _: P, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.func_map(f)
    }
}

impl<A, B, const N: usize> FuncMap<A, B> for [A; N] {
    type Output = [B; N];

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<A, B> FuncMap<A, B> for Cell<A> {
    type Output = Cell<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        f(self.into_inner()).into()
    }
}

impl<A, B> FuncMap<A, B> for Option<A> {
    type Output = Option<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<A, B> FuncMap<A, B> for PhantomData<A> {
    type Output = PhantomData<B>;

    fn func_map<F>(self, _: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        PhantomData
    }
}

impl<A, B, E> FuncMap<A, B> for Result<A, E> {
    type Output = Result<B, E>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<T, A, B> FuncMap<A, B, TypeParam<1>> for Result<T, A> {
    type Output = Result<T, B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map_err(f)
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    extern crate alloc;

    use alloc::boxed::Box;
    use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap};
    use alloc::vec::Vec;

    use super::*;

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
}
