use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod dependency;
mod ident_collector;
mod iter;
mod mapstruct;
mod struct_mapper;
mod type_ext;

// TODO detect crate name?
// TODO check auto-impl
// TODO use tinyvec etc.?
// TODO bounds on original type
//   --> take care of mapped param on rhs of bound: Foo<T: Into<T>>, Foo<T, S: Into<T>>
//   --> take care of where clauses
// TODO enums
// TODO use fuzzing tests?
// TODO rename fmap/func_map
// TODO docs (deny missing docs)
// TODO trybuild tests
// TODO deny some lints (missing docs, missing debug)
// TODO impl more standard types (HashMap, ...)
// TODO allow attributes on generated impl
// TODO allow restricting which params should be mappable
// TODO reduce usage of parse_quote!(..)
// TODO merge where clauses with identical left-hand sides?

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
