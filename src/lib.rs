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
