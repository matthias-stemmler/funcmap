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
// TODO docs
// TODO unit tests
// TODO deny some lints (missing docs)
// TODO impl types from popular crates?
// TODO MSRV policy?
// TODO no_std test (-> Serde)
// TODO GitHub Actions: cargo msrv --verify, cargo nono check, dependabot (see Serde)
// TODO pub -> pub(crate) (https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unreachable-pub) (or not -> see Clippy)
// TODO MIRI test for unsafe

#[proc_macro_derive(FuncMap, attributes(funcmap))]
pub fn derive_func_map(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    match derive::derive_func_map(derive_input) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    }
    .into()
}
