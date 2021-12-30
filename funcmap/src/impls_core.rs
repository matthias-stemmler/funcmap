use crate::{FuncMap, TypeParam};

use core::cell::Cell;
use core::marker::PhantomData;

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

impl<A, B, E> FuncMap<A, B, TypeParam<0>> for Result<A, E> {
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
