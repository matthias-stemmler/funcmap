//! Derivable functorial mappings for Rust
//!
//! This crate provides the [`FuncMap`] trait that can be automatically derived
//! for a type that is generic over a type parameter. It then applies a given
//! closure to all occurrences of the type parameter within the type, thus
//! avoiding a lot of boilerplate mapping code.
//!
//! Concretely, given a generic type `Foo<T>` and an `FnMut(A) -> B` closure,
//! it can turn any value of type `Foo<A>` into a value of type `Foo<B>`. This
//! is called a *functorial mapping* following the *functor* design pattern of
//! functional programming.
//!
//! # Basic Usage
//!
//! Suppose you have a type that is generic over some type parameter `T` and
//! contains a `T` in various places:
//! ```
//! struct Foo<T> {
//!     value: T,
//!     more_values: Vec<T>,
//! }
//! ```
//!
//! Now suppose you want to turn a `Foo<i32>` into a `Foo<String>` by converting
//! each [`i32`] contained in the type into a [`String`] by applying
//! [`to_string()`](ToString::to_string). You can do this by deriving the
//! [`FuncMap`] trait provided by this crate and then invoking its
//! [`func_map`](FuncMap::func_map) method like this:
//! ```
//! # use funcmap::FuncMap;
//! #[derive(FuncMap)]
//! struct Foo<T> {
//!     value: T,
//!     more_values: Vec<T>,
//! }
//!
//! let foo = Foo {
//!     value: 1,
//!     more_values: vec![2, 3, 4],
//! };
//!
//! let bar = foo.func_map(|v| v.to_string());
//!
//! assert_eq!(bar.value, "1");
//! assert_eq!(bar.more_values, vec!["2", "3", "4"]);
//! ```
//!
//! The expression `foo.func_map(|v| v.to_string())` is equivalent to this:
//! ```
//! # use funcmap::FuncMap;
//! # #[derive(FuncMap, Clone, Debug, PartialEq)]
//! # struct Foo<T> {
//! #     value: T,
//! #     more_values: Vec<T>,
//! # }
//! #
//! # let foo = Foo {
//! #     value: 1,
//! #     more_values: vec![2, 3, 4],
//! # };
//! #
//! # let foo_orig = foo.clone();
//! #
//! # let bar =
//! Foo {
//!     value: foo.value.to_string(),
//!     more_values: foo.more_values.into_iter().map(|v| v.to_string()).collect()
//! }
//! #
//! # ;
//! #
//! # assert_eq!(foo_orig.func_map(|v| v.to_string()), bar);
//! ```
//!
//! This way, you can avoid some amount of boilerplate mapping code, especially
//! if your type contains many and/or deeply nested occurrences of `T`.
//!
//! This works for both structs and enums and many ways of nesting `T` within
//! your type such as arrays, tuples and many types from the standard library as
//! well as your own types as long as they derive [`FuncMap`] themselves.
//!
//! For a more detailed explanation and more features, see the following
//! sections.
//!
//! # Common Use Cases
//!
//! Refer to examples
//!
//! # How It Works
//!
//! The [`FuncMap`] derive macro supports both structs and enums. For structs,
//! both tuple structs and structs with named fields are supported. When mapping
//! an enum, the variant will stay the same while the variant's fields will be
//! mapped just like the fields of a struct.
//!
//! Suppose you're deriving [`FuncMap`] for a type that is generic over `T` and
//! then applying [`funcmap`](FuncMap::func_map) to a value of that type,
//! providing an `FnMut(A) -> B` closure:
//! ```
//! # use funcmap::FuncMap;
//! #[derive(FuncMap)]
//! struct Foo<T> {
//!     // ...
//!     # _value: T,
//! }
//!
//! let foo = Foo {
//!     // ...
//!     # _value: ()
//! };
//!
//! let bar = foo.func_map(|v| { /* ... */});
//! ```
//!
//! Then any field of `Foo<T>` whose type doesn't depend on `T` is left
//! untouched.
//!
//! For any field whose type depends on `T`, the following types are supported:
//! * arrays: `[T1; N]`, where `T1` is a type depending on `T`
//! * tuples of arbitrary length: `(T1, ..., Tn)` where at least one of the `Ti`
//!   depends on `T`
//! * named generic types: `Bar<T1, ..., Tn>` where at least one of the `Ti`
//!   depends on `T`
//!
//! In the case of a named generic type, the derived implementation of
//! [`FuncMap`] for `Foo<T>` has the appropriate trait bounds to allow for
//! recursive application of [`func_map`](FuncMap::func_map) on
//! `Bar<T1, ..., Tn>`. In order to fulfill these trait bounds,
//! `Bar<T1, ..., Tn>` must satisfy one of these conditions:
//! * It is a type from the standard library for which this crate provides an
//!   implementation of [`FuncMap`], such as [`Vec<T>`].
//! * It is a type defined in your crate for which [`FuncMap`] is derived.
//! * It is a type defined in your crate for which you implement [`FuncMap`]
//!   manually.
//!
//! Other types depending on `T` such as references (e.g. `&'a T`) or function
//! pointers (e.g. `fn() -> T`) are not supported. This doesn't mean that `T`
//! itself cannot be a reference type (it can), but just that it cannot occur
//! behind a reference within `Foo<T>`.
//!
//! # Fallible Mappings
//!
//! The closure you pass to the
//! [`func_map`](FuncMap::func_map) method must not fail. If you have a closure
//! that can fail, you can use the [`try_func_map`](FuncMap::try_func_map)
//! method instead. This method takes a closure returning a [`Result<B, E>`] for
//! some error type `E` and returns a result with the same error type `E`:
//! ```
//! # use funcmap::FuncMap;
//! # use std::num::{IntErrorKind, ParseIntError};
//! # #[derive(Debug)]
//! #[derive(FuncMap)]
//! struct Foo<T> {
//!     value1: T,
//!     value2: T,
//!     value3: T,
//! }
//!
//! let foo = Foo {
//!     value1: "42", // can be parsed as i32
//!     value2: "1a", // cannot be parsed as i32 -> IntErrorKind::InvalidDigit
//!     value3: "",   // cannot be parsed as i32 -> IntErrorKind::Empty
//! };
//!
//! let bar: Result<Foo<i32>, ParseIntError> = foo.try_func_map(|v| v.parse());
//!
//! assert!(bar.is_err());
//! assert_eq!(*bar.unwrap_err().kind(), IntErrorKind::InvalidDigit);
//! ```
//!
//! As you can see in the example, when there are multiple errors,
//! [`try_func_map`](FuncMap::try_func_map) returns the first one according to
//! the order of the fields in the definition of `Foo<T>`.
//!
//! # Multiple Type Parameters
//!
//! [`TypeParam`] marker, for config refer to config section
//! Recommend `const` alias for [`TypeParam`]
//! `func_map_over`
//!
//! # Configuration
//!
//! `crate`, `params`
//!
//! # Manually Implementing [`FuncMap`]
//!
//! Implement only `try_func_map`, others are provided
//!
//! # `no_std`
//!
//! Explain how to use with `#![no_std]`
//!
//! # Features
//!
//! Mention `alloc` and `std`
//!
//! # Functional Programming Background
//!
//! Functors, category theory, Haskell, fmap
//!
//! # MSRV

#![cfg_attr(not(feature = "std"), no_std)]
// Builtin lints
#![deny(warnings)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
// #![deny(missing_docs)] // TODO uncomment
#![deny(unreachable_pub)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]
#![deny(unused_lifetimes)]
#![deny(unused_qualifications)]
// Clippy lints
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::multiple_crate_versions)]
#![deny(clippy::rest_pat_in_fully_bound_structs)]
#![deny(clippy::use_debug)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // TODO remove
// Rustdoc lints
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::private_doc_tests)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_rust_codeblocks)]
#![deny(rustdoc::bare_urls)]

mod array;
mod impls_core;

#[cfg(feature = "alloc")]
mod impls_alloc;

#[cfg(feature = "std")]
mod impls_std;

use core::convert::Infallible;
use core::fmt::{self, Display, Formatter};

/// Functorial mapping of a generic type over any of its type parameters
pub trait FuncMap<A, B, P = TypeParam<0>>: Sized {
    type Output;

    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>;

    fn try_func_map_over<Q, E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
        Q: Equals<P>,
    {
        self.try_func_map(f)
    }

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.try_func_map::<Infallible, _>(|value| Ok(f(value)))
            .unwrap()
    }

    fn func_map_over<Q, F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
        Q: Equals<P>,
    {
        self.func_map(f)
    }
}

/// Derive macro generating an impl of the trait [`FuncMap`]
pub use funcmap_derive::FuncMap;

/// Marker type specifying one of multiple type parameters to map over
///
/// The const generic `N` is the zero-based index of the type parameter, not
/// counting lifetime parameters[^const-generics].
///
/// For example, for a type `Foo<'a, S, T>`,
/// - [`TypeParam<0>`] refers to `S` and
/// - [`TypeParam<1>`] refers to `T`.
///
/// [^const-generics]: While lifetime parameters are explicitly not counted,
/// this is not relevant for const generics as they must be declared *after*
/// type parameters and hence do not affect the indices of type parameters.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeParam<const N: usize> {}

impl<const N: usize> Display for TypeParam<N> {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

/// Marker trait for type equality
///
/// For any two types `P` and `Q`, the trait bound `P: Equals<Q>` is satisfied
/// if and only if `P == Q`.
///
/// This trait is sealed and cannot be implemented outside of `funcmap`.
pub trait Equals<T>: private::Sealed<T> {}

// Note that from `Q: Equals<P>`
// - if `Q` is known, then the compiler can infer `P`
// - if `P` is known, then the compiler *cannot* infer `Q`
//
// This way, we force the user to make `Q` explicit when using
// [`FuncMap::func_map_over`] because that is the whole purpose of this method.
// If `Q` could be inferred, then it wouldn't be needed and using
// [`FuncMap::func_map`] would be more idiomatic.
impl<T> Equals<T> for T {}

mod private {
    pub trait Sealed<T> {}

    impl<T> Sealed<T> for T {}
}
