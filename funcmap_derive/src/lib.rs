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
// TODO impl more standard types (HashMap, ...) + (optional) popular crates?
// --> std: HashMap (+IntoIter), HashSet (+IntoIter), BufReader, BufWriter, Chain, Cursor, LineWriter, Take, Empty, Mutex, RwLock
// TODO allow more lints? (native ones are handled, consider clippy)
// TODO MSRV policy?
// TODO no_std test (-> Serde)
// TODO GitHub Actions: cargo msrv --verify, cargo nono check, dependabot (see Serde)
// TODO bugfix: idents starting with r# cause panic
// TODO pub -> pub(crate) (https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unreachable-pub)

#[proc_macro_derive(FuncMap, attributes(funcmap))]
pub fn derive_func_map(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);

    match derive::derive_func_map(derive_input) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    }
    .into()
}
