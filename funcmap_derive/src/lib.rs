//! Derive macros for the `funcmap` crate
//!
//! This crate should not be depended on directly. See the documentation of
//! [funcmap](/funcmap) instead.

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
#![allow(clippy::too_many_lines)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_rust_codeblocks)]
#![deny(rustdoc::bare_urls)]

use derivable::Derivable;

use proc_macro::TokenStream;

mod derivable;
mod derive;
mod ident;
mod ident_collector;
mod input;
mod map;
mod opts;
mod predicates;
mod result;
mod syn_ext;

/// Derive macro generating implementations of the `FuncMap` trait
#[proc_macro_derive(FuncMap, attributes(funcmap))]
pub fn derive_func_map(item: TokenStream) -> TokenStream {
    derive::derive(item.into(), Derivable::Standard).into()
}

/// Derive macro generating implementations of the `TryFuncMap` trait
#[proc_macro_derive(TryFuncMap, attributes(funcmap))]
pub fn derive_try_func_map(item: TokenStream) -> TokenStream {
    derive::derive(item.into(), Derivable::Fallible).into()
}
