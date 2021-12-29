use std::fmt::{self, Display, Formatter};

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;

pub const CRATE_IDENT: StaticIdent = StaticIdent("funcmap");
pub const TRAIT_IDENT: StaticIdent = StaticIdent("FuncMap");
pub const FN_IDENT: StaticIdent = StaticIdent("func_map");
pub const FN_IDENT_WITH_MARKER: StaticIdent = StaticIdent("func_map_over");
pub const OUTPUT_TYPE_IDENT: StaticIdent = StaticIdent("Output");
pub const MARKER_TYPE_IDENT: StaticIdent = StaticIdent("TypeParam");
pub const ATTR_IDENT: StaticIdent = StaticIdent("funcmap");

#[derive(Debug)]
pub struct StaticIdent(&'static str);

impl StaticIdent {
    pub fn to_ident(&self) -> Ident {
        Ident::new(self.0, Span::call_site())
    }
}

impl AsRef<str> for StaticIdent {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Display for StaticIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToTokens for StaticIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_ident().to_tokens(tokens)
    }
}
