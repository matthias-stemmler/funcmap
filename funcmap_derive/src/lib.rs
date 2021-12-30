#![deny(missing_debug_implementations)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod derive;
mod error;
mod ident_collector;
mod idents;
mod input;
mod map_expr;
mod opts;
mod predicates;
mod syn_ext;

// TODO check auto-impl/structopt/serde
// TODO use fuzzing tests?
// TODO docs
// TODO unit tests
// TODO deny some lints (missing docs)
// TODO impl more standard types (HashMap, ...) + (optional) popular crates?
// TODO allow more lints?
// TODO MSRV policy?
// TODO expand tests (see serde)

// TODO CI: cargo msrv --verify, cargo nono check

#[proc_macro_derive(FuncMap, attributes(funcmap))]
pub fn derive_func_map(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    match derive::derive_func_map(derive_input) {
        Ok(output) => {
            diagnostic::print(&output);
            output
        }
        Err(err) => err.to_compile_error(),
    }
    .into()
}

#[cfg(feature = "debug")]
mod diagnostic;

#[cfg(not(feature = "debug"))]
mod diagnostic {
    pub fn print(_: &proc_macro2::TokenStream) {}
}