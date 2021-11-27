use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;

pub const CRATE_IDENT: StaticIdent = StaticIdent("funcmap");
pub const TRAIT_IDENT: StaticIdent = StaticIdent("FuncMap");
pub const FN_IDENT: StaticIdent = StaticIdent("func_map");
pub const FN_IDENT_WITH_MARKER: StaticIdent = StaticIdent("func_map_over");
pub const OUTPUT_TYPE_IDENT: StaticIdent = StaticIdent("Output");
pub const MARKER_TYPE_IDENT: StaticIdent = StaticIdent("TypeParam");

pub struct StaticIdent(&'static str);

impl ToTokens for StaticIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Ident::new(self.0, Span::call_site()).to_tokens(tokens);
    }
}
