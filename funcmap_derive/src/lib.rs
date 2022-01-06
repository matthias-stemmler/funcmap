#![deny(warnings)]
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
// #![deny(clippy::cargo_common_metadata)] // TODO uncomment
#![deny(clippy::multiple_crate_versions)]
#![deny(clippy::rest_pat_in_fully_bound_structs)]
#![deny(clippy::use_debug)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // TODO remove
#![allow(clippy::too_many_lines)]

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
// TODO impl types from popular crates?
// TODO MSRV policy?
// TODO no_std test (-> Serde)
// TODO GitHub Actions: cargo msrv --verify, cargo nono check, dependabot (see Serde)
// TODO MIRI test for unsafe
// TODO resolve TODOs

#[proc_macro_derive(FuncMap, attributes(funcmap))]
pub fn derive_func_map(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    match derive::derive_func_map(derive_input) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    }
    .into()
}
