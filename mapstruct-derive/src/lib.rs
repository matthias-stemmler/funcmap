#![deny(missing_debug_implementations)]

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod ident_collector;
mod map_expr;
mod mapstruct;
mod predicates;
mod syn_ext;

// TODO detect crate name?
// TODO check auto-impl
// TODO enums
// TODO use fuzzing tests?
// TODO rename fmap/func_map
// TODO docs
// TODO trybuild tests
// TODO deny some lints (missing docs)
// TODO impl more standard types (HashMap, ...)
// TODO allow attributes on generated impl
// TODO allow restricting which params should be mappable
// TODO reduce usage of parse_quote!(..)
// TODO newtype?

#[proc_macro_error]
#[proc_macro_derive(MapStruct)]
pub fn derive_map_struct(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);
    let output = mapstruct::derive_map_struct(derive_input);
    diagnostic::print(&output);
    output.into()
}

#[cfg(feature = "debug")]
mod diagnostic;

#[cfg(not(feature = "debug"))]
mod diagnostic {
    pub fn print(_: &proc_macro2::TokenStream) {}
}
