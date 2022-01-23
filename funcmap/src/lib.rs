//! Derivable functorial mappings for Rust
//!
//! This crate provides the [`FuncMap`] trait (and its fallible version
//! [`TryFuncMap`]) that can be automatically derived for a type that is generic
//! over a type parameter. It provides a method that applies a given closure to
//! all (potentially nested) occurrences of the type parameter within the type,
//! removing the need to write verbose mapping code.
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
//! This way, you avoid writing boilerplate mapping code, especially in cases
//! where your type contains many and/or deeply nested occurrences of `T`.
//!
//! This works for both structs and enums and many ways of nesting `T` within
//! your type such as arrays, tuples and many types from the standard library as
//! well as your own types as long as they implement [`FuncMap`] themselves.
//!
//! For a more detailed explanation and more features, see the following
//! sections. Everything stated about [`FuncMap`] applies to [`TryFuncMap`]
//! as well unless stated otherwise.
//!
//! # Common Use Cases
//!
//! Refer to examples
//!
//! # How It Works
//!
//! The [`FuncMap`] trait has two required parameters (and one optional
//! parameter, see below) that refer to the source and target type,
//! respectively, of the closures to be used as mapping functions. The
//! associated type [`Output`](FuncMap::Output) defines the overall output type
//! of the mapping.
//!
//! Concretely, if you derive [`FuncMap`] for a type `Foo<T>`, then
//! ```
//! # use funcmap::FuncMap;
//! #
//! # #[derive(FuncMap)]
//! # struct Foo<T>(T);
//! #
//! # enum A {}
//! # enum B {}
//! #
//! # fn test() where
//! Foo<A>: FuncMap<A, B, Output = Foo<B>>
//! # {}
//! ```
//! holds for any two types `A` and `B`. The choice of `A` and `B` is only
//! restricted by any trait bounds on `T` in the definition of `Foo<T>` and
//! [`FuncMap`] trait bounds needed for the mapping of inner types, see below.
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
//! First of all, any field of `Foo<T>` whose type doesn't depend on `T` is left
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
//! [`FuncMap`] for `Foo<T>` carries the appropriate trait bounds to allow for
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
//! The closure passed to the [`func_map`](FuncMap::func_map) method must not
//! fail. If you have a closure that can fail, you can use the [`TryFuncMap`]
//! trait and its method [`try_func_map`](TryFuncMap::try_func_map) instead.
//! [`TryFuncMap`] can be derived in the same way and or the same types as
//! [`FuncMap`]. Since [`FuncMap`] is a supertrait of [`TryFuncMap`], make sure
//! you derive [`FuncMap`] as well.
//!
//! The [`try_func_map`](TryFuncMap::try_func_map) method takes a
//! closure returning a [`Result<B, E>`] for some error type `E` and returns a
//! result with the same error type `E`:
//! ```
//! # use funcmap::{FuncMap, TryFuncMap};
//! # use std::num::{IntErrorKind, ParseIntError};
//! # #[derive(Debug)]
//! #[derive(FuncMap, TryFuncMap)]
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
//! [`try_func_map`](TryFuncMap::try_func_map) returns the first one according
//! to the order of the fields in the definition of `Foo<T>`.
//!
//! # Multiple Type Parameters
//!
//! When a type is generic over multiple type parameters, then the [`FuncMap`]
//! derive macro will by default generate separate implementations for mapping
//! over each type parameter.
//!
//! This can create an ambiguity that is resolved by using the [`TypeParam`]
//! marker type as a third parameter to [`FuncMap`] to specify which type
//! parameter to map over.
//!
//! To see why this is necessary, consider a type `Foo<S, T>` with two type
//! parameters. Then there are two ways of applying an `FnMut(A) -> B` closure
//! to the type `Foo<A, A>`:
//! - mapping over the type parameter `S`, producing a `Foo<B, A>`
//! - mapping over the type parameter `T`, producing a `Foo<A, B>`
//!
//! Since both cannot be handled by a single implementation of `FuncMap<A, B>`
//! for `Foo<A>`, the [`FuncMap`] trait has a third parameter `P` to distinguish
//! between the two. This parameter is instantiated with the types
//! `TypeParam<0>` and `TypeParam<1>`, respectively, so that
//! ```
//! # use funcmap::{FuncMap, TypeParam};
//! #
//! # #[derive(FuncMap)]
//! # struct Foo<S, T>(S, T);
//! #
//! # enum A {}
//! # enum B {}
//! #
//! # fn test() where
//! Foo<A, A>: FuncMap<A, B, TypeParam<0>, Output = Foo<B, A>>,
//! Foo<A, A>: FuncMap<A, B, TypeParam<1>, Output = Foo<A, B>>
//! # {}
//! ```
//! This distinction is done purely on the type system level, so
//! [`TypeParam<const N: usize>`] is a pure marker type of which no values
//! exist. The number `N` specifies the 0-based index of the type parameter to
//! map over. If the type has any lifetime parameters, they are not counted, so
//! even for `Foo<'a, S, T>`,
//! - `TypeParam<0>` refers to `S`
//! - `TypeParam<1>` refers to `T`
//!
//! The `P` parameter of [`FuncMap`] defaults to `TypeParam<0>`, so it can be
//! ignored completely in case there is only a single type parameter.
//!
//! Note that when calling [`func_map`](FuncMap::func_map), the correct type
//! for `P` can often be inferred:
//! ```
//! # use funcmap::FuncMap;
//! # #[derive(Debug, PartialEq)]
//! #[derive(FuncMap)]
//! struct Foo<S, T> {
//!     s: S,
//!     t: T,
//! }
//!
//! let foo = Foo { s: 42, t: "Hello" };
//!
//! // Here `P` is inferred as `TypeParam<1>`
//! let bar = foo.func_map(ToString::to_string);
//! #
//! # assert_eq!(bar, Foo { s: 42, t: String::from("Hello") });
//! ```
//!
//! When it cannot be inferred, it can be cumbersome to specify explicitly
//! because it's the *trait* [`FuncMap`] that is generic over `P`, not its
//! method [`func_map`](FuncMap::func_map). To mitigate this, the [`FuncMap`]
//! trait has another method [`func_map_over`](FuncMap::func_map_over) that does
//! exactly the same thing as [`func_map`](FuncMap::func_map) but allows you to
//! specify the type parameter marker `P` explicitly:
//! ```
//! # use funcmap::{FuncMap, TypeParam};
//! #
//! # #[derive(Copy, Clone)]
//! #[derive(FuncMap, Debug, PartialEq)]
//! struct Foo<S, T> {
//!     s: S,
//!     t: T,
//! }
//!
//! let foo = Foo { s: 42, t: 42 };
//!
//! let bar = foo.func_map_over::<TypeParam<1>, _>(|x| x + 1);
//! // Equivalent to: FuncMap::<_, _, TypeParam<1>>::func_map(foo, |x| x + 1);
//! // This would be ambiguous: let bar = foo.func_map(|x| x + 1);
//!
//! assert_eq!(bar, Foo { s: 42, t: 43 });
//! # assert_eq!(
//! #     FuncMap::<_, _, TypeParam<1>>::func_map(foo, |x| x + 1),
//! #     Foo { s: 42, t: 43 }
//! # );
//! ```
//!
//! Note that you need to write `func_map_over::<TypeParam<1>, _>` rather than
//! just `func_map_over::<TypeParam<1>>` because
//! [`func_map_over`](FuncMap::func_map_over) has a second parameter that is the
//! type of the given closure.
//!
//! To improve readability and make your code more robust to changes, it is
//! advisable to define type aliases for the markers that convey the meaning of
//! the corresponding types and abstract away their concrete indices:
//! ```
//! # use funcmap::{FuncMap, TypeParam};
//! #
//! type WidthParam = TypeParam<0>;
//! type HeightParam = TypeParam<1>;
//!
//! #[derive(FuncMap, Debug, PartialEq)]
//! struct Size<W, H> {
//!     width: W,
//!     height: H
//! }
//!
//! let normal = Size { width: 100, height: 100 };
//! let skewed = normal
//!     .func_map_over::<WidthParam, _>(|w| w * 2)
//!     .func_map_over::<HeightParam, _>(|h| h * 3);
//!
//! assert_eq!(skewed, Size { width: 200, height: 300 });
//! ```
//!
//! By default, implementations for all type parameters are generated. You can
//! restrict this to only a subset of the type parameters by configuration as
//! described in the next section. This becomes necessary if any of the type
//! parameters occur within the type in a way that's not supported by the
//! [`FuncMap`] derive macro.
//!
//! # Customizing derive behavior
//!
//! When deriving [`FuncMap`] or [`TryFuncMap`] for a type, you can change the
//! default behavior of the derive macro through the optional `#[funcmap]`
//! helper attribute. This attribute may only be applied to the type itself, not
//! to its fields or variants:
//! ```
//! # use funcmap as my_funcmap;
//! # use funcmap::{FuncMap, TryFuncMap};
//! #[derive(FuncMap, TryFuncMap)]
//! #[funcmap(crate = "my_funcmap", params(S, T))] // options are explained below
//! struct Foo<S, T, U> {
//!     value1: S,
//!     value2: T,
//!     value3: U,
//! }
//! ```
//!
//! Options can also be put into separate `#[funcmap]` attributes, so the
//! following is equivalent:
//! ```
//! # use funcmap as my_funcmap;
//! # use funcmap::{FuncMap, TryFuncMap};
//! #[derive(FuncMap, TryFuncMap)]
//! #[funcmap(crate = "my_funcmap")]
//! #[funcmap(params(S))]
//! #[funcmap(params(T))]
//! struct Foo<S, T, U> {
//!     value1: S,
//!     value2: T,
//!     value3: U,
//! }
//! ```
//!
//! Note that this way of customizing the derive macro doesn't distinguish
//! between [`FuncMap`] and [`TryFuncMap`]. The options are always the same for
//! both.
//!
//! The following options are available:
//!
//! ## `#[funcmap(crate = "...")`
//!
//! This defines the path to the `funcmap` crate instance to use when referring
//! to `funcmap` APIs from generated implementations. This will only be needed
//! in rare cases, e.g. when you rename `funcmap` in the `dependencies` section
//! of your `Cargo.toml` or invoke a re-exported `funcmap` derive in a public
//! macro.
//!
//! ## `#[funcmap(params(...))]`
//!
//! If the annotated type has
//! [multiple type parameters](#multiple-type-parameters), this defines for
//! which of the type parameters an implementation should be generated by
//! providing a comma-separated list of type parameters. If the `params` option
//! is omitted, the default behavior is that implementations for *all* type
//! parameters are generated.
//!
//! This is especially useful if you need to exclude a type parameter because it
//! occurs within the annotated type in a way unsuitable for deriving `FuncMap`:
//! ```
//! # use funcmap::FuncMap;
//! #[derive(FuncMap)]
//! #[funcmap(params(S, T))]
//! struct Foo<'a, S, T, U> {
//!     value: S,
//!     more_values: Vec<T>,
//!     reference: &'a U,
//! }
//! ```
//!
//! Here, without the line `#[funcmap(params(S, T))]`, the [`FuncMap`] derive
//! macro would try to generate implementations for all three type parameters
//! `S`, `T` and `U` and fail because `U` occurs within `Foo` behind a
//! reference, which is not supported, see [How It Works](#how-it-works).
//!
//! The `params` option can also be used to decrease compile time when a
//! `FuncMap` implementation for some type parameter is not needed.
//!
//! # Manually Implementing [`FuncMap`] and [`TryFuncMap`]
//!
//! # `no_std` support
//!
//! `funcmap` has a Cargo feature named `std` that is enabled by default and
//! provides implementations of [`FuncMap`] and [`TryFuncMap`] for many types
//! from the [`std`] library. In order to use `funcmap` in a `no_std` context,
//! modify your dependency on `funcmap` in `Cargo.toml` to opt out of default
//! features:
//! ```toml
//! [dependencies]
//! funcmap = { version = "...", default-features = false }
//! ```
//!
//! In this case, only implementations for types in the [`core`] library are
//! provided. Note that this excludes implementations for all standard library
//! types that involve heap memory allocation, such as [`Box<T>`] or [`Vec<T>`].
//! In order to opt back in to these implementations, you can enable the `alloc`
//! Cargo feature:
//! ```toml
//! [dependencies]
//! funcmap = { version = "...", default-features = false, features = ["alloc"] }
//! ```
//!
//! This will provide implementatios for many types in the [`alloc`] library.
//!
//! # Functional Programming Background
//!
//! Functors, category theory, Haskell, fmap
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
// #![deny(missing_docs)] // TODO uncomment
#![deny(unreachable_pub)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]
#![deny(unused_lifetimes)]
#![deny(unused_qualifications)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::multiple_crate_versions)]
#![deny(clippy::rest_pat_in_fully_bound_structs)]
#![deny(clippy::use_debug)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // TODO remove
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

use core::fmt::{self, Display, Formatter};

/// Functorial mapping of a generic type over any of its type parameters
pub trait FuncMap<A, B, P = TypeParam<0>>: Sized {
    type Output;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B;

    fn func_map_over<Q, F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
        Q: Equals<P>,
    {
        self.func_map(f)
    }
}

/// Fallible functorial mapping of a generic type over any of its type
/// parameters
pub trait TryFuncMap<A, B, P = TypeParam<0>>: FuncMap<A, B, P> {
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
}

/// Derive macro generating an impl of the trait [`FuncMap`]
pub use funcmap_derive::FuncMap;

/// Derive macro generating an impl of the trait [`TryFuncMap`]
pub use funcmap_derive::TryFuncMap;

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
