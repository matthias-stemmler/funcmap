//! [`Derivable`] type for for managing different derivable traits

use crate::ident::{
    StaticIdent, FALLIBLE_FN_IDENT, FALLIBLE_TRAIT_IDENT, FN_IDENT,
    NO_DROP_MARKER_FALLIBLE_TRAIT_IDENT, NO_DROP_MARKER_TRAIT_IDENT, TRAIT_IDENT,
};

use proc_macro2::TokenStream;
use quote::quote;

/// A derivable trait
#[derive(Copy, Clone, Debug)]
pub(crate) enum Derivable {
    /// The derivable trait `FuncMap`
    Standard,

    /// The derivable trait `TryFuncMap`
    Fallible,
}

impl Derivable {
    /// Returns the trait identifier of this derivable trait
    pub(crate) fn trait_ident(self) -> StaticIdent {
        match self {
            Self::Standard => TRAIT_IDENT,
            Self::Fallible => FALLIBLE_TRAIT_IDENT,
        }
    }

    /// Returns the identifier of the "no drop" marker trait corresponding to
    /// this derivable trait
    pub(crate) fn no_drop_marker_trait_ident(self) -> StaticIdent {
        match self {
            Self::Standard => NO_DROP_MARKER_TRAIT_IDENT,
            Self::Fallible => NO_DROP_MARKER_FALLIBLE_TRAIT_IDENT,
        }
    }

    /// Returns the identifier of the required method of this derivable trait
    pub(crate) fn fn_ident(self) -> StaticIdent {
        match self {
            Self::Standard => FN_IDENT,
            Self::Fallible => FALLIBLE_FN_IDENT,
        }
    }

    /// "Unwraps" an expression in implementations of this derivable trait
    ///
    /// This is named after the *bind* operation for monads
    pub(crate) fn bind_expr(self, expr: TokenStream) -> TokenStream {
        match self {
            Self::Standard => expr,
            Self::Fallible => quote!(#expr?),
        }
    }

    /// "Wraps" an expression in implementations of this derivable trait
    ///
    /// This named after the *unit* operation for monads
    pub(crate) fn unit_expr(self, expr: TokenStream) -> TokenStream {
        match self {
            Self::Standard => expr,
            Self::Fallible => quote!(::core::result::Result::Ok(#expr)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::{parse_quote, Expr};

    #[test]
    fn bind_for_standard_returns_expression_unchanged() {
        let bound = Derivable::Standard.bind_expr(quote!(value));

        let expr: Result<Expr, _> = syn::parse2(bound);

        assert!(expr.is_ok());
        assert_eq!(expr.unwrap(), parse_quote!(value));
    }

    #[test]
    fn bind_for_fallible_wraps_expression_with_question_mark() {
        let bound = Derivable::Fallible.bind_expr(quote!(value));

        let expr: Result<Expr, _> = syn::parse2(bound);

        assert!(expr.is_ok());
        assert_eq!(expr.unwrap(), parse_quote!(value?));
    }

    #[test]
    fn unit_for_standard_returns_expression_unchanged() {
        let bound = Derivable::Standard.unit_expr(quote!(value));

        let expr: Result<Expr, _> = syn::parse2(bound);

        assert!(expr.is_ok());
        assert_eq!(expr.unwrap(), parse_quote!(value));
    }

    #[test]
    fn unit_for_fallible_wraps_expression_in_ok() {
        let bound = Derivable::Fallible.unit_expr(quote!(value));

        let expr: Result<Expr, _> = syn::parse2(bound);

        assert!(expr.is_ok());
        assert_eq!(
            expr.unwrap(),
            parse_quote!(::core::result::Result::Ok(value))
        );
    }
}
