#![deny(missing_debug_implementations)]

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod derive;
mod ident_collector;
mod idents;
mod map_expr;
mod predicates;
mod syn_ext;

// TODO check if attributes need to be carried over (within block)
// TODO detect crate name?
// TODO check auto-impl/structopt/serde
// TODO use fuzzing tests?
// TODO docs
// TODO trybuild tests
// TODO deny some lints (missing docs)
// TODO impl more standard types (HashMap, ...)
// TODO allow attributes on generated impl
// TODO allow restricting which params should be mappable

// TODO set span to call_site on idents (and other tokens?) taken from input
// TODO check unsized types
// TODO for<'a> T: 'a

#[proc_macro_error]
#[proc_macro_derive(FuncMap)]
pub fn derive_func_map(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);
    let output = derive::derive_func_map(derive_input);
    diagnostic::print(&output);
    output.into()
}

#[cfg(feature = "debug")]
mod diagnostic;

#[cfg(not(feature = "debug"))]
mod diagnostic {
    pub fn print(_: &proc_macro2::TokenStream) {}
}
