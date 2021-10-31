#![no_std]

use core::marker::PhantomData;

#[doc(hidden)]
pub use mapstruct_derive::*;

pub struct TypeParam<const N: usize>;

pub trait MapStruct<A, B, P = TypeParam<0>>: Sized {
    type Output;

    fn map_struct<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B;

    fn map_struct_over<F>(self, _: P, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map_struct(f)
    }
}

impl<A, B, const N: usize> MapStruct<A, B> for [A; N] {
    type Output = [B; N];

    fn map_struct<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<A, B> MapStruct<A, B> for Option<A> {
    type Output = Option<B>;

    fn map_struct<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<A, B, E> MapStruct<A, B> for Result<A, E> {
    type Output = Result<B, E>;

    fn map_struct<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<T, A, B> MapStruct<A, B, TypeParam<1>> for Result<T, A> {
    type Output = Result<T, B>;

    fn map_struct<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map_err(f)
    }
}

impl<A, B> MapStruct<A, B> for PhantomData<A> {
    type Output = PhantomData<B>;

    fn map_struct<F>(self, _: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        PhantomData
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    extern crate alloc;

    use alloc::boxed::Box;
    use alloc::vec::Vec;

    use super::*;

    impl<A, B> MapStruct<A, B> for Box<A> {
        type Output = Box<B>;

        fn map_struct<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            Box::new(f(*self))
        }
    }

    impl<A, B> MapStruct<A, B> for Vec<A> {
        type Output = Vec<B>;

        fn map_struct<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.into_iter().map(f).collect()
        }
    }
}

struct Foo<S, T> {
    s: S,
    t: T,
}

impl<A, B, T> MapStruct<A, B, TypeParam<0>> for Foo<A, T> {
    type Output = Foo<B, T>;

    fn map_struct<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        Self::Output {
            s: f(self.s),
            t: self.t,
        }
    }
}

impl<S, A, B> MapStruct<A, B, TypeParam<1>> for Foo<S, A> {
    type Output = Foo<S, B>;

    fn map_struct<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        Self::Output {
            s: self.s,
            t: f(self.t),
        }
    }
}

struct Bar<U, V> {
    u: U,
    v: V,
}

impl<A, B, V> MapStruct<A, B, TypeParam<0>> for Bar<A, V> {
    type Output = Bar<B, V>;

    fn map_struct<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        Self::Output {
            u: f(self.u),
            v: self.v,
        }
    }
}

impl<U, A, B> MapStruct<A, B, TypeParam<1>> for Bar<U, A> {
    type Output = Bar<U, B>;

    fn map_struct<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        Self::Output {
            u: self.u,
            v: f(self.v),
        }
    }
}
