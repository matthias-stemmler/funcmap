#![no_std]
#![deny(missing_debug_implementations)]

mod impls_core;

#[cfg(feature = "alloc")]
mod impls_alloc;

#[cfg(feature = "std")]
mod impls_std;

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
