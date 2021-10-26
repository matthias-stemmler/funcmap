use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};

mod depends_on;
mod ident_collector;
mod iter;
mod macros;
mod mapstruct;
mod path;
mod struct_mapper;
mod subs_type_param;
mod type_nesting;

// TODO use proc_macro_error
// TODO detect crate name?
// TODO check auto-impl
// TODO use tinyvec etc.?

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
