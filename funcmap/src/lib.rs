#![no_std]
#![deny(missing_debug_implementations)]

mod array;
mod impls_core;

#[cfg(feature = "alloc")]
mod impls_alloc;

#[cfg(feature = "std")]
mod impls_std;

use core::{convert::Infallible, hint};

#[doc(hidden)]
pub use funcmap_derive::*;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeParam<const N: usize>;

pub trait FuncMap<A, B, P = TypeParam<0>>: Sized {
    type Output;

    fn try_func_map<F, E>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>;

    fn try_func_map_over<F, E>(self, _: P, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.try_func_map(f)
    }

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        let result: Result<_, Infallible> = self.try_func_map(|value| Ok(f(value)));

        match result {
            Ok(value) => value,

            // SAFETY: `_err` is of type `Infallible`, of which no values exist, so this arm is never reached
            Err(_err) => unsafe { hint::unreachable_unchecked() },
        }
    }

    fn func_map_over<F>(self, _: P, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.func_map(f)
    }
}
