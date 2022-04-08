# funcmap - Derivable functorial mappings for Rust

[![GitHub](https://img.shields.io/badge/GitHub-informational?logo=GitHub&labelColor=555555)](https://github.com/matthias-stemmler/funcmap)
[![crates.io](https://img.shields.io/crates/v/funcmap.svg)](https://crates.io/crates/funcmap)
[![docs.rs](https://img.shields.io/docsrs/funcmap)](https://docs.rs/funcmap/latest/funcmap/)
[![license](https://img.shields.io/crates/l/funcmap.svg)](https://github.com/matthias-stemmler/funcmap/blob/main/LICENSE-APACHE)
[![rustc 1.56+](https://img.shields.io/badge/rustc-1.56+-lightgrey.svg)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)

This Rust crate provides the [`FuncMap`] (and its fallible version
[`TryFuncMap`]) that can be automatically derived for a type that is generic
over a type parameter. It provides a method that applies a given closure to all
(potentially nested) occurrences of the type parameter within the type, removing
the need to write verbose mapping code.

Concretely, given a generic type `Foo<T>` and an `FnMut(A) -> B` closure, it can
turn any value of type `Foo<A>` into a value of type `Foo<B>`. This is called a
_functorial mapping_ following the _functor_ design pattern of functional
programming.

## Installation

This crate is available on [crates.io](https://crates.io/crates/funcmap). In
order to use it, add this to the `dependencies` table of your `Cargo.toml`:

```toml
[dependencies]
funcmap = "0.1.0"
```

## Usage

Suppose you have a type that is generic over some type parameter `T` and
contains a `T` in various places:

```rust
struct Foo<T> {
    value: T,
    more_values: Vec<T>,
}
```

Now suppose you want to turn a `Foo<i32>` into a `Foo<String>` by converting
each `i32` contained in the type into a `String` by applying `to_string`. You
can do this by deriving the [`FuncMap`] trait provided by this crate and then
invoking its [`func_map`] method like this:

```rust
#[derive(FuncMap)]
struct Foo<T> {
    value: T,
    more_values: Vec<T>,
}

let foo = Foo {
    value: 1,
    more_values: vec![2, 3, 4],
};

let bar = foo.func_map(|v| v.to_string());

assert_eq!(bar.value, "1");
assert_eq!(bar.more_values, vec!["2", "3", "4"]);
```

The expression `foo.func_map(|v| v.to_string())` is equivalent to this:

```rust
Foo {
    value: foo.value.to_string(),
    more_values: foo.more_values.into_iter().map(|v| v.to_string()).collect()
}
```

This way, you avoid writing boilerplate mapping code, especially in cases where
your type contains many and/or deeply nested occurrences of `T`. This works for
both structs and enums and many ways of nesting `T` within your type such as
arrays, tuples and many types from the standard library as well as your own
types as long as they implement [`FuncMap`] themselves. Note that the purpose of
the `funcmap` crate is just to provide utility functionality, so

- you shouldn't depend on any of the items it exports in your public API,
- it shouldn't be necessary to use bounds on the traits it exports anywhere
  except in generic implementations of those same traits.

For a more detailed explanation and more features, see the
[crate documentation](https://docs.rs/funcmap/latest/funcmap/).

For larger examples, see the [examples](funcmap/examples) folder.

## Minimum Supported Rust Version (MSRV) policy

The current MSRV of this crate is `1.56`.

Increasing the MSRV of this crate is _not_ considered a breaking change.
However, in such cases there will be at least a minor version bump. Each version
of this crate will support at least the four latest stable Rust versions at the
time it is published.

## Changelog

See [CHANGELOG.md](CHANGELOG.md)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[`funcmap`]: https://docs.rs/funcmap/latest/funcmap/trait.FuncMap.html
[`tryfuncmap`]: https://docs.rs/funcmap/latest/funcmap/trait.TryFuncMap.html
[`func_map`]: https://docs.rs/funcmap/latest/funcmap/trait.FuncMap.html#tymethod.func_map
