//! [![GitHub](https://img.shields.io/badge/GitHub-informational?logo=GitHub&labelColor=555555)](https://github.com/matthias-stemmler/funcmap)
//! [![crates.io](https://img.shields.io/crates/v/funcmap.svg)](https://crates.io/crates/funcmap)
//! [![docs.rs](https://img.shields.io/docsrs/funcmap)](https://docs.rs/funcmap/latest/funcmap/)
//! [![license](https://img.shields.io/crates/l/funcmap.svg)](https://github.com/matthias-stemmler/funcmap/blob/main/LICENSE-APACHE)
//! [![rustc 1.56+](https://img.shields.io/badge/rustc-1.56+-lightgrey.svg)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)
//!
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
//! [`to_string`](ToString::to_string). You can do this by deriving the
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
//! Note that the purpose of the `funcmap` crate is just to provide utility
//! functionality, so
//! - you shouldn't depend on any of the items it exports in your public API,
//! - it shouldn't be necessary to use bounds on the traits it exports anywhere
//!   except in generic implementations of those same traits.
//!
//! For a more detailed explanation and more features, see the following
//! sections. Everything stated about [`FuncMap`] applies to [`TryFuncMap`]
//! as well unless mentioned otherwise.
//!
//! For larger examples, see the `examples` folder in the crate repository.
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
//! [`FuncMap`] and [`Sized`] trait bounds needed for the mapping of inner
//! types, see below.
//!
//! The [`FuncMap`] derive macro supports both structs and enums. For structs,
//! both tuple structs and structs with named fields are supported. When mapping
//! an enum, the variant stays the same while the variant's fields are mapped
//! just like the fields of a struct.
//!
//! Suppose you derive [`FuncMap`] for a type that is generic over `T` and then
//! apply [`funcmap`](FuncMap::func_map) to a value of that type, providing an
//! `FnMut(A) -> B` closure:
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
//! * arrays: `[T0; N]`, where `T0` is a type depending on `T`
//! * tuples of arbitrary length: `(T0, ..., Tn)` where at least one of the `Ti`
//!   depends on `T`
//! * named generic types: `Bar<T0, ..., Tn>` where at least one of the `Ti`
//!   depends on `T`
//!
//! In the case of a named generic type, the derived implementation of
//! [`FuncMap`] for `Foo<T>` carries the appropriate trait bounds to allow for
//! recursive application of [`func_map`](FuncMap::func_map) on
//! `Bar<T0, ..., Tn>`. In order to fulfill these trait bounds,
//! `Bar<T0, ..., Tn>` must satisfy one of these conditions:
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
//! You can have a look at the code generated by the [`FuncMap`] derive macro
//! by using [`cargo-expand`](https://github.com/dtolnay/cargo-expand).
//!
//! ## Caveats
//!
//! ### [`FuncMap`] trait bounds
//!
//! When deriving [`FuncMap`] for a type `Foo<T>` that has a field of type
//! `Bar<T>`, where `Bar<T>` *doesn't* implement [`FuncMap`], the derive macro
//! won't fail, nor will it just *assume* that `Bar<T>` implements [`FuncMap`],
//! which would cause a compile error within the derived implementation.
//!
//! The reason is that the derive macro cannot know whether `Bar<T>` implements
//! [`FuncMap`] and it needs to deal with the fact that `Bar<T>` could implement
//! [`FuncMap`] for *some* types `T` while it doesn't implement it for others.
//!
//! So what it does instead is to add an appropriate trait bound to the derived
//! implementation that looks like this:
//! ```
//! # use funcmap::FuncMap;
//! #
//! # struct Foo<T>(Bar<T>);
//! # struct Bar<T>(T);
//! #
//! impl<A, B> FuncMap<A, B> for Foo<A>
//! where
//!     Bar<A>: FuncMap<A, B, Output = Bar<B>>
//! {
//!     type Output = Foo<B>;
//!
//!     // ...
//! #   fn func_map<F>(self, f: F) -> Self::Output
//! #   where
//! #       F: FnMut(A) -> B
//! #   {
//! #       Foo(self.0.func_map(f))
//! #   }
//! }
//! ```
//! This trait bound on `Bar<A>` puts an implicit condition on `A` and `B`. More
//! precisely, `Foo<A>` implements [`FuncMap<A, B>`] only for those `A` and `B`
//! where `Bar<A>` also implements [`FuncMap<A, B>`]. If `Bar<T>` doesn't
//! implement [`FuncMap`] *at all*, this condition is never satisfied. In this
//! case, the derived implementation still compiles but doesn't add any
//! functionality.
//!
//! ### [`Sized`] trait bounds
//!
//! The trait [`FuncMap<A, B>`] puts [`Sized`] bounds on the type parameters `A`
//! and `B` as well as any type `Foo<A>` it is implemented for and the
//! corresponding output type `Foo<B>`.
//!
//! Derived implementations of [`FuncMap`] additionally require the types of all
//! the fields of `Foo<A>` and `Foo<B>` to be [`Sized`]. In the case of a type
//! depending on `A` (in `Foo<A>`) or `B` (in the output), this is implicit in
//! the [`FuncMap`] trait bounds mentioned in the previous section. For types
//! that don't depend on `A` or `B`, the [`FuncMap`] derive macro adds an
//! explicit [`Sized`] bound to the derived implementation.
//!
//! This is again because a field could have a type `Bar<T>` that is generic
//! over another type parameter `T` different from `A` and `B` and `Bar<T>`
//! could be [`Sized`] for *some* `T` but not for others. So the implementation
//! applies only to those types `T` where all the fields are [`Sized`].
//!
//! ### Types implementing [`Drop`]
//!
//! Deriving [`FuncMap`] is only possible for types that do not implement
//! [`Drop`] because the derived implementation for a type needs to move out of
//! the fields of the type, which isn't possible for [`Drop`] types. Trying to
//! derive [`FuncMap`] for types implementing [`Drop`] leads to a compile error.
//! (Strictly speaking, it would technically be possible if all the fields were
//! [`Copy`], but in this case it would very likely make no sense anyway for the
//! reasons described
//! [here](https://doc.rust-lang.org/std/marker/trait.Copy.html#when-cant-my-type-be-copy),
//! so it is still disallowed.)
//!
//! However, if a type `Foo<T>` implements [`Drop`], you can still implement
//!  [`FuncMap`] for `Foo<T>` manually. For instance, in the case where all the
//! fields of `Foo<T>` have types implementing [`Default`], you can move out of
//! the fields using [`core::mem::take`] like this:
//! ```
//! use funcmap::FuncMap;
//!
//! // cannot `#[derive(FuncMap)]` because `Foo<T>: Drop`
//! struct Foo<T> {
//!     value: T,
//! }
//!
//! impl<T> Drop for Foo<T> {
//!     fn drop(&mut self) {
//!         // apply some cleanup logic
//!     }
//! }
//!
//! impl<A, B> FuncMap<A, B> for Foo<A>
//! where
//!     A: Default,
//! {
//!     type Output = Foo<B>;
//!
//!     fn func_map<F>(mut self, mut f: F) -> Self::Output
//!     where
//!         F: FnMut(A) -> B,
//!     {
//!         Foo {
//!             value: f(core::mem::take(&mut self.value)),
//!         }
//!     }
//! }
//! ```
//! In case a field of `Foo<T>` has a type `Bar<T>` that doesn't implement
//! [`Default`], it may be possible to replace it with `Option<Bar<T>>`, which
//! implements [`Default`].
//!
//! # Fallible Mappings
//!
//! The closure passed to the [`func_map`](FuncMap::func_map) method must not
//! fail. If you have a closure that can fail, you can use the [`TryFuncMap`]
//! trait and its method [`try_func_map`](TryFuncMap::try_func_map) instead.
//! [`TryFuncMap`] can be derived in the same way and for the same types as
//! [`FuncMap`].
//!
//! The [`try_func_map`](TryFuncMap::try_func_map) method takes a
//! closure returning a [`Result<B, E>`] for some error type `E` and returns a
//! result with the same error type `E`:
//! ```
//! # use funcmap::TryFuncMap;
//! # use std::num::{IntErrorKind, ParseIntError};
//! # #[derive(Debug)]
//! #[derive(TryFuncMap)]
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
//! [`TypeParam<0>`] and [`TypeParam<1>`], respectively, so that
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
//! Note that while lifetime parameters aren't counted, const generics are. The
//! reason for this is that when the derive macro looks at arguments of nested
//! types, it may not be able to distinguish const arguments from type arguments
//! syntactically. So, for `Foo<'a, const N: usize, S, const M: usize, T>`,
//! - `TypeParam<1>` refers to `S`
//! - `TypeParam<3>` refers to `T`
//! and `TypeParam<0>` and `TypeParam<2>` are not used at all.
//!
//! The `P` parameter of [`FuncMap`] defaults to `TypeParam<0>`, so it can be
//! ignored completely in case there is only a single type parameter, at least
//! if it's not preceded by a const generic.
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
//! recommended to define type aliases for the markers that convey the meaning
//! of the corresponding types and abstract away their concrete indices:
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
//! ## Caveat: Type aliases
//!
//! Suppose a type `Foo<T>` has a field whose type has multiple type parameters:
//! ```
//! # #[derive(funcmap::FuncMap)]
//! # struct Bar<T, U, V>(T, U, V);
//! #
//! # #[derive(funcmap::FuncMap)]
//! struct Foo<T> {
//!     value: Bar<T, i32, T>,
//! }
//! ```
//!
//! Then the derived implementation of [`FuncMap`] for `Foo<T>` delegates to the
//! [`FuncMap`] implementation of `Bar<T, U, V>` using the marker types
//! [`TypeParam<N>`], where `N` is the 0-based index of the respective type
//! parameter of `Bar<T, U, V>`. In the example, `Bar<T, i32, T>` will be mapped
//! using
//! - [`TypeParam<0>`] to map over the first instance of `T`,
//! - [`TypeParam<2>`] to map over the second instance of `T`.
//!
//! Now if `Bar<T, U, V>` happens to be an alias for a type where `T`, `U` and
//! `V` appear at different positions within its list of type parameters, this
//! will not work. For instance, if
//! ```
//! # struct Baz<T, U, V, W>(T, U, V, W);
//! type Bar<T, U, V> = Baz<i32, V, U, T>;
//! ```
//!
//! then a [`FuncMap`] implementation of the right-hand side will map over `T`,
//! say, using [`TypeParam<3>`], not [`TypeParam<0>`].
//!
//! Consequently, when deriving [`FuncMap`] for a type whose definition uses
//! type aliases, make sure to follow the
//!
//! **Rule:** Every type parameter of the alias (or at least the ones that are
//! instantiated with a type parameter over which [`FuncMap`] is derived) must
//! have the same index among the type parameters of the alias as within the
//! type parameters of the type the alias stands for.
//!
//! Remember that lifetime parameters are not counted, so this is fine, for
//! example:
//! ```
//! # struct Baz<'a, T>(&'a T);
//! type Bar<T> = Baz<'static, T>;
//! ```
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
//! If a type has [multiple type parameters](#multiple-type-parameters), this
//! defines for which of the type parameters an implementation should be
//! generated by providing a comma-separated list of type parameters. If the
//! `params` option is omitted, the default behavior is that implementations for
//! *all* type parameters are generated.
//!
//! This is especially useful if you need to exclude a type parameter because it
//! occurs within a type in a way unsuitable for deriving `FuncMap`:
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
//! Even though implementations of the traits in this crate are usually meant to
//! be derived automatically, it can become necessary for you to implement the
//! traits manually in some cases, for instance
//! - when a type in your crate has a field depending on a type parameter in a
//!   way that isn't supported by the [`FuncMap`] and [`TryFuncMap`] derive
//!   macros, e.g. when you implement a low-level primitive such as your custom
//!   version of [`Vec<T>`],
//! - when you need a [`FuncMap`] or [`TryFuncMap`] implementation for a type in
//!   a third-party crate that doesn't provide one.
//!
//! In the latter case, since you cannot implement nor derive the trait for a
//! third-party type due to the orphan rule, you can provide your own wrapper
//! around it (following the *newtype* pattern) and implement the trait manually
//! for the wrapper type:
//! ```
//! # use funcmap::FuncMap;
//! #
//! // Pretend that this is an external crate, not a module
//! mod third_party {
//!     # #[derive(Debug, PartialEq)]
//!     pub struct List<T> {
//!         // ...
//!         # pub value: Option<T>,
//!     }
//!
//!     impl<A> List<A> {
//!         pub fn map<B>(self, f: impl FnMut(A) -> B) -> List<B> {
//!             // ...
//!             # List { value: self.value.map(f) }
//!         }
//!     }
//! }
//!
//! // In your crate:
//! # #[derive(Debug, PartialEq)]
//! struct MyList<T>(third_party::List<T>);
//!
//! impl<A, B> FuncMap<A, B> for MyList<A> {
//!     type Output = MyList<B>;
//!
//!     fn func_map<F>(self, f: F) -> Self::Output
//!     where
//!         F: FnMut(A) -> B,
//!     {
//!         MyList(self.0.map(f))
//!     }
//! }
//!
//! // Now you can derive `FuncMap` for types containing a `MyList<T>`:
//! # #[derive(Debug, PartialEq)]
//! #[derive(FuncMap)]
//! struct Foo<T> {
//!     list: MyList<T>,
//! }
//! #
//! # let list = Foo {
//! #     list: MyList(third_party::List {
//! #         value: Some(1)
//! #     })
//! # };
//! #
//! # assert_eq!(
//! #     list.func_map(|v| v.to_string()),
//! #     Foo {
//! #         list: MyList(third_party::List {
//! #             value: Some(String::from("1"))
//! #         })
//! #     }
//! # );
//! ```
//!
//! For details on the exact contract to uphold when writing manual
//! implementations, see the API documentations of [`FuncMap`] and
//! [`TryFuncMap`].
//!
//! Note that if you have already implemented [`TryFuncMap`] for a type, you can
//! then always implement [`FuncMap`] like this:
//! ```
//! use funcmap::{FuncMap, TryFuncMap};
//!
//! # #[derive(Debug, PartialEq)]
//! struct Foo<T> {
//!     // ...
//! #   value: T,
//! }
//!
//! impl<A, B> TryFuncMap<A, B> for Foo<A> {
//!     type Output = Foo<B>;
//!
//!     // ...
//! #
//! #   fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
//! #   where
//! #       F: FnMut(A) -> Result<B, E>,
//! #   {
//! #       Ok(Foo {
//! #           value: f(self.value)?
//! #       })
//! #   }
//! }
//!
//! impl<A, B> FuncMap<A, B> for Foo<A> {
//!     type Output = Foo<B>;
//!
//!     fn func_map<F>(self, mut f: F) -> Self::Output
//!     where
//!         F: FnMut(A) -> B,
//!     {
//!         self.try_func_map::<std::convert::Infallible, _>(|x| Ok(f(x))).unwrap()
//!     }
//! }
//! #
//! # let foo = Foo { value: 42 };
//! # assert_eq!(foo.func_map(|x| x + 1), Foo { value: 43 });
//! ```
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
//! This will provide implementations for many types in the [`alloc`] library.
//!
//! # Functional Programming Background
//!
//! The idea of `funcmap` is based on the *functor* design pattern from
//! functional programming, which in turn is inspired from the notion of a
//! functor in category theory.
//!
//! Basically, `F` is a functor if
//! 1. it associates each type `T` with a new type `F(T)`
//! 2. it associates each function
//!     ```plain
//!     f: A -> B
//!     ```
//!     with a function
//!     ```plain
//!     F(f): F(A) -> F(B)
//!     ```
//!     such that the following *functor laws* are satisfied:
//!     - `F(id) = id` where `id` is the identity function on `A`, respectively
//!       `F(A)`
//!     - `F(g . f) = F(g) . F(f)` for any two functions `f: A -> B` and
//!       `g: B -> C`, where `g . f` denotes function composition
//!
//! In languages with higher-kinded types such as Haskell, this property of
//! being a functor is expressed as a *type class* (similar to a trait) called
//! [Functor](https://wiki.haskell.org/Functor) that the higher-kinded type `F`
//! is an instance of.
//!
//! In Rust, property 1. is satisfied for every type `Foo<T>` that is generic
//! over a type parameter `T` because it associates each type `T` with a new
//! type `Foo<T>`, at least for those types `T` that satisfy all trait bounds
//! that `Foo<T>` imposes on `T`.
//!
//! Property 2. is where the [`FuncMap`] trait comes into play. As there are no
//! higher-kinded types in Rust as of now, it cannot be expressed by `Foo`
//! itself implementing a trait, because while `Foo<T>` is a type for every `T`,
//! `Foo` itself (without the `<T>`) isn't something one can reason about within
//! the Rust type system. However, one can say that `Foo` is a functor if and
//! only if
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
//! holds for all types `A` and `B` for which `Foo<T>` exists. The function
//! `Foo<A> -> Foo<B>` associated with a function `f: A -> B` (in Rust:
//! `f: impl FnMut(A) -> B`) by property 2. is then provided by the
//! [`func_map`](FuncMap::func_map) method as the function
//! ```
//! # use funcmap::FuncMap;
//! #
//! # #[derive(FuncMap)]
//! # struct Foo<T>(T);
//! #
//! # enum A {}
//! # enum B {}
//! #
//! # fn test(f: impl FnMut(A) -> B) -> impl FnOnce(Foo<A>) -> Foo<B> {
//! |x: Foo<A>| x.func_map(f)
//! # }
//! ```
//! So deriving the [`FuncMap`] trait for `Foo<T>` can be viewed as deriving
//! Property 2. from Property 1. or equivalently, deriving a (hypothetical)
//! *Functor*  trait for the (hypothetical) higher-kinded type `Foo`.
//!
//! In fact, the name of the [`func_map`](FuncMap::func_map) method is inspired
//! from the
//! [`fmap`](https://hackage.haskell.org/package/base-4.16.0.0/docs/Data-Functor.html#v:fmap)
//! function of Haskell's `Functor` type class.
//!
//! # Minimum Supported Rust Version (MSRV) policy
//!
//! The current MSRV of this crate is `1.56`.
//!
//! Increasing the MSRV of this crate is *not* considered a breaking change.
//! However, in such cases there will be at least a minor version bump.
//!
//! Each version of this crate will support at least the four latest stable Rust
//! versions at the time it is published.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
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
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_rust_codeblocks)]
#![deny(rustdoc::bare_urls)]

mod array;
mod impls_core;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod impls_alloc;

#[cfg(feature = "std")]
mod impls_std;

use core::fmt::{self, Display, Formatter};

/// Functorial mapping of a generic type over any of its type parameters
///
/// # Deriving [`FuncMap`]
///
/// In most cases, implementations of this trait can and should be derived
/// automatically:
/// ```
/// # use funcmap::FuncMap;
/// #
/// #[derive(FuncMap)]
/// struct Foo<T> {
///     // ...
///     # value: T,
/// }
/// ```
///
/// See the [crate-level documentation](crate) for details.
///
/// # Manually implementing [`FuncMap`]
///
/// If you need to implement [`FuncMap`] manually, make sure to uphold the
/// following contract:
///
/// Let `Foo` be a type that is generic over the type or const parameters
/// `T0, ..., Tn`.
///
/// If `Foo` implements [`FuncMap<A, B, TypeParam<N>>`], then
/// - `N` must be in the range `0..=n`.
/// - The parameter of `Foo` at index `N` (not counting lifetime parameters)
///   must be `A`. In particular, it must be a type parameter, not a const
///   generic.
/// - `Foo::Output` must be `Foo` with the parameter at index `N` replaced with
///   `B`.
///
/// Furthermore:
/// - [`func_map_over`](Self::func_map_over) must behave in exactly the same way
///   as [`func_map`](Self::func_map). This is the default behavior and must not
///   be changed.
/// - When implementing [`FuncMap`] for different marker types [`TypeParam<N>`]
///   and [`TypeParam<M>`], the result of mapping over both type parameters in
///   sequence must not depend on the order of the two mappings, i.e.
///   ```
///   # use funcmap::{FuncMap, TypeParam};
///   #
///   # #[derive(FuncMap, Copy, Clone, Debug, PartialEq)]
///   # struct Foo<T, U>(T, U);
///   #
///   # const N: usize = 0;
///   # const M: usize = 1;
///   #
///   # let foo = Foo(42, 43);
///   # let f = |x| x + 1;
///   # let g = |x| x * 2;
///   #
///   # assert!(
///   foo.func_map_over::<TypeParam<N>, _>(f)
///      .func_map_over::<TypeParam<M>, _>(g)
///
///   // must be equivalent to
///   # ==
///
///   foo.func_map_over::<TypeParam<M>, _>(g)
///      .func_map_over::<TypeParam<N>, _>(f)  
///   # );
///   ```
pub trait FuncMap<A, B, P = TypeParam<0>>: Sized
where
    P: FuncMarker<P>,
{
    /// The output type of the functorial mapping
    ///
    /// This is `Self` with the parameter at index `N` replaced with `B`, where
    /// `N` is such that `P` is `TypeParam<N>`.
    ///
    /// In the simplest case of a type with just a single type parameter, if
    /// `Self` is `Foo<A>`, then this is `Foo<B>`.
    type Output;

    /// Applies the closure `f` to `self` in a functorial way
    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B;

    /// Applies the closure `f` to `self` in a factorial way, allowing explicit
    /// specification of the marker type `P`
    ///
    /// This is a convenience method that has the exact same functionality as
    /// [`func_map`](Self::func_map) but can be used to specify the marker type
    /// `P` in a convenient way in cases where it is ambiguous.
    ///
    /// So if you have
    /// ```
    /// # use funcmap::FuncMap;
    /// #
    /// #[derive(FuncMap, Debug, PartialEq)]
    /// struct Foo<S, T> {
    ///     s: S,
    ///     t: T,
    /// }
    ///
    /// let foo = Foo {
    ///     s: 42,
    ///     t: 42,
    /// };
    /// ```
    /// then instead of writing
    /// ```
    /// # use funcmap::{FuncMap, TypeParam};
    /// #
    /// # #[derive(FuncMap, Debug, PartialEq)]
    /// # struct Foo<S, T> {
    /// #     s: S,
    /// #     t: T,
    /// # }
    /// #
    /// # let foo = Foo {
    /// #     s: 42,
    /// #     t: 42,
    /// # };
    /// #
    /// let bar = FuncMap::<_, _, TypeParam<1>>::func_map(foo, |v| v + 1);
    /// assert_eq!(bar, Foo { s: 42, t: 43 });
    /// ```
    ///
    /// you can more conveniently write
    ///
    /// ```
    /// # use funcmap::{FuncMap, TypeParam};
    /// #
    /// # #[derive(FuncMap, Debug, PartialEq)]
    /// # struct Foo<S, T> {
    /// #     s: S,
    /// #     t: T,
    /// # }
    /// #
    /// # let foo = Foo {
    /// #     s: 42,
    /// #     t: 42,
    /// # };
    /// #
    /// let bar = foo.func_map_over::<TypeParam<1>, _>(|v| v + 1);
    /// assert_eq!(bar, Foo { s: 42, t: 43 });
    /// ```
    ///
    /// This lets you chain method calls more easily as in
    /// ```
    /// # use funcmap::{FuncMap, TypeParam};
    /// #
    /// # #[derive(FuncMap, Debug, PartialEq)]
    /// # struct Foo<S, T> {
    /// #     s: S,
    /// #     t: T,
    /// # }
    /// #
    /// # let foo = Foo {
    /// #     s: 42,
    /// #     t: 42,
    /// # };
    /// #
    /// foo.func_map_over::<TypeParam<0>, _>(|v| v + 1)
    ///    .func_map_over::<TypeParam<1>, _>(|v| v + 1)
    /// # ;
    /// ```
    ///
    /// Note that you still need to specify the inferred type `_` for the
    /// closure type `F`.
    fn func_map_over<Q, F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
        Q: FuncMarker<P>,
    {
        self.func_map(f)
    }
}

/// Fallible functorial mapping of a generic type over any of its type
/// parameters
///
/// # Deriving [`TryFuncMap`]
///
/// In most cases, implementations of this trait can and should be derived
/// automatically:
/// ```
/// # use funcmap::TryFuncMap;
/// #
/// #[derive(TryFuncMap)]
/// struct Foo<T> {
///     // ...
///     # value: T,
/// }
/// ```
///
/// See the [crate-level documentation](crate) for details.
///
/// # Manually implementing [`TryFuncMap`]
///
/// If you need to implement [`TryFuncMap`] manually, make sure to uphold the
/// following contract:
///
/// Let `Foo` be a type that is generic over the type or const parameters
/// `T0, ..., Tn`.
///
/// If `Foo` implements [`TryFuncMap<A, B, TypeParam<N>>`], then
/// - `N` must be in the range `0..=n`.
/// - The parameter of `Foo` at index `N` (not counting lifetime parameters)
///   must be `A`. In particular, it must be a type parameter, not a const
///   generic.
/// - `Foo::Output` must be `Foo` with the parameter at index `N` replaced with
///   `B`.
///
/// Furthermore:
/// - [`try_func_map_over`](Self::try_func_map_over) must behave in exactly the
///   same way as [`try_func_map`](Self::try_func_map). This is the default
///   behavior and must not be changed.
/// - If the closure provided to [`try_func_map`](Self::try_func_map) fails,
///   then the result must be the first error according to the order of the
///   fields in the definition of `Foo`:
///   ```
///   # use funcmap::TryFuncMap;
///   # use std::num::{IntErrorKind, ParseIntError};
///   #
///   #[derive(TryFuncMap, Copy, Clone, Debug, PartialEq)]
///   struct Foo<T> {
///       value1: T,
///       value2: T,
///   }
///   
///   let foo = Foo {
///       value1: "1a",
///       value2: ""
///   };
///
///   let result: Result<Foo<i32>, ParseIntError> = foo.try_func_map(|v| v.parse());
///   
///   assert!(result.is_err());
///   assert_eq!(*result.unwrap_err().kind(), IntErrorKind::InvalidDigit);
///   ```
/// - When implementing [`TryFuncMap`] for different marker types
///   [`TypeParam<N>`] and [`TypeParam<M>`], the result of mapping over both
///   type parameters in sequence must not depend on the order of the two
///   mappings, i.e.
///   ```
///   # use funcmap::{TryFuncMap, TypeParam};
///   #
///   # #[derive(TryFuncMap, Copy, Clone, Debug, PartialEq)]
///   # struct Foo<T, U>(T, U);
///   #
///   # const N: usize = 0;
///   # const M: usize = 1;
///   #
///   # let foo = Foo(42, 43);
///   # let f = |x| Ok::<_, ()>(x + 1);
///   # let g = |x| Ok::<_, ()>(x * 2);
///   #
///   # assert!(
///   foo.try_func_map_over::<TypeParam<N>, _, _>(f)
///      .and_then(|x| x.try_func_map_over::<TypeParam<M>, _, _>(g))
///
///   // must be equivalent to
///   # ==
///
///   foo.try_func_map_over::<TypeParam<M>, _, _>(g)
///      .and_then(|x| x.try_func_map_over::<TypeParam<N>, _, _>(f))
///   # );
///   ```
pub trait TryFuncMap<A, B, P = TypeParam<0>>: Sized
where
    P: FuncMarker<P>,
{
    /// The output type of the functorial mapping
    ///
    /// This is `Self` with the parameter at index `N` replaced with `B`, where
    /// `N` is such that `P` is `TypeParam<N>`.
    ///
    /// In the simplest case of a type with just a single type parameter, if
    /// `Self` is `Foo<A>`, then this is `Foo<B>`.
    type Output;

    /// Tries to apply the closure `f` to `self` in a functorial way
    ///
    /// # Errors
    /// Fails if and only if `f` fails, returning the first error according to
    /// the order of the fields in the definition of `Self`
    fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>;

    /// Tries to apply the closure `f` to `self` in a factorial way, allowing
    /// explicit specification of the marker type `P`
    ///
    /// This is a convenience method that has the exact same functionality as
    /// [`try_func_map`](Self::try_func_map) but can be used to specify the
    /// marker type `P` in a convenient way in cases where it is ambiguous.
    ///
    /// So if you have
    /// ```
    /// # use funcmap::TryFuncMap;
    /// #
    /// #[derive(TryFuncMap, Debug, PartialEq)]
    /// struct Foo<S, T> {
    ///     s: S,
    ///     t: T,
    /// }
    ///
    /// let foo = Foo {
    ///     s: "42",
    ///     t: "42",
    /// };
    /// ```
    /// then instead of writing
    /// ```
    /// # use funcmap::{TryFuncMap, TypeParam};
    /// #
    /// # #[derive(TryFuncMap, Debug, PartialEq)]
    /// # struct Foo<S, T> {
    /// #     s: S,
    /// #     t: T,
    /// # }
    /// #
    /// # let foo = Foo {
    /// #     s: "42",
    /// #     t: "42",
    /// # };
    /// #
    /// let bar = TryFuncMap::<_, _, TypeParam<1>>::try_func_map(foo, |v| v.parse::<i32>());
    /// assert_eq!(bar, Ok(Foo { s: "42", t: 42 }));
    /// ```
    ///
    /// you can more conveniently write
    ///
    /// ```
    /// # use funcmap::{TryFuncMap, TypeParam};
    /// #
    /// # #[derive(TryFuncMap, Debug, PartialEq)]
    /// # struct Foo<S, T> {
    /// #     s: S,
    /// #     t: T,
    /// # }
    /// #
    /// # let foo = Foo {
    /// #     s: "42",
    /// #     t: "42",
    /// # };
    /// #
    /// let bar = foo.try_func_map_over::<TypeParam<1>, _, _>(|v| v.parse::<i32>());
    /// assert_eq!(bar, Ok(Foo { s: "42", t: 42 }));
    /// ```
    ///
    /// This lets you chain method calls more easily as in
    /// ```
    /// # use funcmap::{TryFuncMap, TypeParam};
    /// #
    /// # #[derive(TryFuncMap, Debug, PartialEq)]
    /// # struct Foo<S, T> {
    /// #     s: S,
    /// #     t: T,
    /// # }
    /// #
    /// # let foo = Foo {
    /// #     s: "42",
    /// #     t: "42",
    /// # };
    /// #
    /// foo.try_func_map_over::<TypeParam<0>, _, _>(|v| v.parse::<i32>())
    ///     .and_then(|foo| foo.try_func_map_over::<TypeParam<1>, _, _>(|v| v.parse::<i32>()))
    /// # ;
    /// ```
    ///
    /// Note that you still need to specify the inferred type `_` for the
    /// error type `E` and the closure type `F`.
    ///
    /// # Errors
    /// Fails if and only if `f` fails, returning the first error according to
    /// the order of the fields in the definition of `Self`
    fn try_func_map_over<Q, E, F>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
        Q: FuncMarker<P>,
    {
        self.try_func_map(f)
    }
}

pub use funcmap_derive::FuncMap;

pub use funcmap_derive::TryFuncMap;

/// Marker type specifying one of multiple type parameters to map over
///
/// The const generic `N` is the zero-based index of the type parameter, not
/// counting lifetime parameters, but counting const generics.
///
/// For example, for a type `Foo<'a, S, T>`,
/// - [`TypeParam<0>`] refers to `S` and
/// - [`TypeParam<1>`] refers to `T`
/// and for a type `Foo<'a, const N: usize, S, const M: usize, T>`,
/// - [`TypeParam<1>`] refers to `S` and
/// - [`TypeParam<3>`] refers to `T`
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeParam<const N: usize> {}

impl<const N: usize> Display for TypeParam<N> {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

/// Marker trait for marker types specifying what to map over
///
/// This is only implemented by the marker types [`TypeParam<N>`] and is used to
/// restrict the choice of types for the `P` type parameter of
/// [`FuncMap<A, B, P>`] and [`TryFuncMap<A, B, P>`].
///
/// Note that [`FuncMarker<P>`] is itself generic over `P` and for all
/// implementations, the type parameter `P` is the implementing type itself.
/// This way, from `Q: FuncMarker<P>` it can be inferred that `Q == P`, which is
/// used in the [`FuncMap::func_map_over`] and [`TryFuncMap::try_func_map_over`]
/// methods.
///
/// This trait is sealed and cannot be implemented outside of `funcmap`.
pub trait FuncMarker<P>: private::Sealed<P> {}

// Note that from `Q: FuncMarker<P>`
// - if `Q` is known, then the compiler can infer `P`,
// - yet if `P` is known, then the compiler *cannot* infer `Q`
//
// This way, we force the user to make `Q` explicit when using
// [`FuncMap::func_map_over`] and [`TryFuncMap::try_func_map_over`] because that
// is the whole purpose of these methods. If `Q` could be inferred, then it
// wouldn't be needed and using [`FuncMap::func_map`] respectively
// [`TryFuncMap::try_func_map`] would be more idiomatic.
impl<const N: usize> FuncMarker<TypeParam<N>> for TypeParam<N> {}

/// Making [`FuncMarker`] a sealed trait
mod private {
    use super::TypeParam;

    /// Private supertrait of [`FuncMarker<P>`](super::FuncMarker)
    pub trait Sealed<P> {}

    impl<const N: usize> Sealed<TypeParam<N>> for TypeParam<N> {}
}

/// Marker trait with a blanket implementation for all types that implement
/// [`Drop`]
///
/// The [`FuncMap`] derive macro produces an implementation of this trait (in
/// addition to an implementation of [`FuncMap`]), asserting that the type
/// doesn't implement [`Drop`] because otherwise there would be conflicting
/// implementations of this trait.
///
/// This is necessary because derived implementation of [`FuncMap`] need to move
/// out of fields, which isn't possible for types implementing [`Drop`].
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub trait FuncMap_cannot_be_derived_for_types_implementing_Drop {}

#[allow(drop_bounds)]
impl<T> FuncMap_cannot_be_derived_for_types_implementing_Drop for T where T: Drop + ?Sized {}

/// Marker trait with a blanket implementation for all types that implement
/// [`Drop`]
///
/// The [`TryFuncMap`] derive macro produces an implementation of this trait (in
/// addition to an implementation of [`TryFuncMap`]), asserting that the type
/// doesn't implement [`Drop`] because otherwise there would be conflicting
/// implementations of this trait.
///
/// This is necessary because derived implementation of [`TryFuncMap`] need to
/// move out of fields, which isn't possible for types implementing [`Drop`].
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub trait TryFuncMap_cannot_be_derived_for_types_implementing_Drop {}

#[allow(drop_bounds)]
impl<T> TryFuncMap_cannot_be_derived_for_types_implementing_Drop for T where T: Drop + ?Sized {}
