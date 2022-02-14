//! Static identifiers referring to items from the `funcmap` crate

use std::fmt::{self, Display, Formatter};

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::Path;

pub(crate) const CRATE_IDENT: StaticIdent = StaticIdent("funcmap");
pub(crate) const TRAIT_IDENT: StaticIdent = StaticIdent("FuncMap");
pub(crate) const FALLIBLE_TRAIT_IDENT: StaticIdent = StaticIdent("TryFuncMap");
pub(crate) const NO_DROP_MARKER_TRAIT_IDENT: StaticIdent =
    StaticIdent("FuncMap_cannot_be_derived_for_types_implementing_Drop");
pub(crate) const NO_DROP_MARKER_FALLIBLE_TRAIT_IDENT: StaticIdent =
    StaticIdent("TryFuncMap_cannot_be_derived_for_types_implementing_Drop");
pub(crate) const FN_IDENT: StaticIdent = StaticIdent("func_map");
pub(crate) const FALLIBLE_FN_IDENT: StaticIdent = StaticIdent("try_func_map");
pub(crate) const OUTPUT_TYPE_IDENT: StaticIdent = StaticIdent("Output");
pub(crate) const MARKER_TYPE_IDENT: StaticIdent = StaticIdent("TypeParam");
pub(crate) const ATTR_IDENT: StaticIdent = StaticIdent("funcmap");

/// A static string slice to be used as an identifier
///
/// This can be turned into a [`TokenStream`] through its [`ToTokens`]
/// implementation. However, unlike [`TokenStream`] itself, it can be
/// constructed in a const context.
#[derive(Copy, Clone, Debug)]
pub(crate) struct StaticIdent(&'static str);

impl StaticIdent {
    /// Turns this [`StaticIdent`] into an actual [`Ident`]
    fn to_ident(self) -> Ident {
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

impl From<StaticIdent> for Path {
    fn from(static_ident: StaticIdent) -> Self {
        static_ident.to_ident().into()
    }
}

impl ToTokens for StaticIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.to_ident().to_tokens(tokens);
    }
}
