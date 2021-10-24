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

#[proc_macro_derive(MapStruct)]
pub fn derive_map_struct(item: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(item as DeriveInput);
    let output = mapstruct::derive_map_struct(derive_input).into();
    eprintln!("{}", output);
    output
}
