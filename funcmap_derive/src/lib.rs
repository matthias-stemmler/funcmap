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
#![allow(clippy::too_many_lines)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::private_doc_tests)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_rust_codeblocks)]
#![deny(rustdoc::bare_urls)]

use derivable::Derivable;

use proc_macro::TokenStream;

mod derivable;
mod derive;
mod error;
mod ident;
mod ident_collector;
mod input;
mod map;
mod opts;
mod predicates;
mod syn_ext;

// TODO check auto-impl/structopt/serde (serde: crates-io.md, html_root_url, explicit "include" in Cargo.toml)
// TODO docs (including root README), resolve TODOs
// TODO unit tests
// TODO MSRV policy?
// TODO no_std test (-> Serde)
// TODO GitHub Actions: cargo msrv --verify, cargo nono check, dependabot (see Serde)
// TODO MIRI test for unsafe
// TODO resolve TODOs
// TODO test publishing (https://github.com/rust-lang/cargo/wiki/Third-party-registries)
// TODO https://rust-lang.github.io/api-guidelines
// TODO https://doc.rust-lang.org/rustdoc/the-doc-attribute.html
// TODO more fallible tests, fallible examples

#[proc_macro_derive(FuncMap, attributes(funcmap))]
pub fn derive_func_map(item: TokenStream) -> TokenStream {
    derive::derive(item.into(), Derivable::Standard).into()
}

#[proc_macro_derive(TryFuncMap, attributes(funcmap))]
pub fn derive_try_func_map(item: TokenStream) -> TokenStream {
    derive::derive(item.into(), Derivable::Fallible).into()
}
