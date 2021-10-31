use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};

mod dependency;
mod ident_collector;
mod iter;
mod macros;
mod mapstruct;
mod path;
mod struct_mapper;
mod subs_type_param;

// TODO use proc_macro_error
// TODO detect crate name?
// TODO check auto-impl
// TODO use tinyvec etc.?
// TODO trait bounds on mappable type
// TODO enums
// TODO use fuzzing tests?
// TODO rename fmap/func_map
// TODO docs (deny missing docs)
// TODO trybuild tests
// TODO deny some lints (missing docs, missing debug)
// TODO optimize `|value| f(value)`to `f`?
// TODO impl more standard types (HashMap, ...)
// TODO allow attributes on generated impl
// TODO allow restricting which params should be mappable
// TODO reduce usage of parse_quote!(..)
// TODO merge where clauses with identical left-hand sides?

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
