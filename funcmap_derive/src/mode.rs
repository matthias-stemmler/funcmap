use proc_macro2::TokenStream;
use quote::quote;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Mode {
    Standard,
    Fallible,
}

impl Mode {
    pub(crate) fn bind(self, expr: TokenStream) -> TokenStream {
        match self {
            Self::Standard => expr,
            Self::Fallible => quote!(#expr?),
        }
    }

    pub(crate) fn unit(self, expr: TokenStream) -> TokenStream {
        match self {
            Self::Standard => expr,
            Self::Fallible => quote!(::core::result::Result::Ok(#expr)),
        }
    }
}
