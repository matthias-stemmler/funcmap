use crate::{error::Error, idents::ATTR_IDENT};

use std::vec;

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Attribute, ConstParam, GenericParam, Lifetime, LifetimeDef, LitStr, Path, Token, TypeParam,
};

mod kw {
    syn::custom_keyword!(params);
}

#[derive(Debug)]
pub struct FuncMapOpts {
    pub crate_path: Option<Path>,
    pub params: Vec<Param>,
}

impl TryFrom<Vec<Attribute>> for FuncMapOpts {
    type Error = Error;

    fn try_from(attrs: Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut crate_path = None;
        let mut params = Vec::new();
        let mut error = Error::new();

        for args_result in attrs
            .into_iter()
            .filter(|attr| attr.path.is_ident(&ATTR_IDENT))
            .map(TryInto::<Args>::try_into)
        {
            match args_result {
                Ok(args) => {
                    for arg in args {
                        match arg {
                            Arg::Crate(ArgCrate(value)) if crate_path.is_none() => {
                                crate_path = Some(value)
                            }

                            Arg::Crate(ArgCrate(value)) => error
                                .combine(syn::Error::new_spanned(value, "duplicate crate path")),

                            Arg::Params(ArgParams(values)) => {
                                for value in values {
                                    if params.contains(&value) {
                                        error.combine(syn::Error::new_spanned(
                                            value,
                                            "duplicate parameter",
                                        ));
                                    } else {
                                        params.push(value);
                                    }
                                }
                            }
                        }
                    }
                }

                Err(err) => error.combine(err),
            }
        }

        error.ok()?;

        Ok(Self { crate_path, params })
    }
}

#[derive(Debug)]
struct Args(Vec<Arg>);

impl TryFrom<Attribute> for Args {
    type Error = Error;

    fn try_from(attr: Attribute) -> Result<Self, Self::Error> {
        Ok(attr.parse_args()?)
    }
}

impl IntoIterator for Args {
    type Item = Arg;
    type IntoIter = vec::IntoIter<Arg>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args: Vec<_> = input
            .call(Punctuated::<_, Token![,]>::parse_terminated)?
            .into_iter()
            .collect();

        if args.is_empty() {
            Err(input.error("expected at least one argument"))
        } else {
            Ok(Self(args))
        }
    }
}

#[derive(Debug)]
enum Arg {
    Crate(ArgCrate),
    Params(ArgParams),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![crate]) {
            Ok(Self::Crate(input.call(ArgCrate::parse)?))
        } else if input.peek(kw::params) {
            Ok(Self::Params(input.call(ArgParams::parse)?))
        } else {
            Err(input.error("expected one of these arguments: `crate`, `params`"))
        }
    }
}

#[derive(Debug)]
struct ArgCrate(Path);

impl Parse for ArgCrate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![crate]>()?;
        input.parse::<Token![=]>()?;
        Ok(Self(input.parse::<LitStr>()?.parse_with(PathParser)?))
    }
}

struct PathParser;

impl Parser for PathParser {
    type Output = Path;

    fn parse2(self, tokens: TokenStream) -> syn::Result<Self::Output> {
        syn::parse2(tokens.clone()).map_err(|_| syn::Error::new_spanned(tokens, "expected path"))
    }
}

#[derive(Debug)]
struct ArgParams(Vec<Param>);

impl Parse for ArgParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::params>()?;
        let content;
        parenthesized!(content in input);
        Ok(Self(content.parse::<TerminatedParams>()?.into()))
    }
}

#[derive(Debug)]
struct TerminatedParams(Vec<Param>);

impl Parse for TerminatedParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.call(Punctuated::<_, Token![,]>::parse_separated_nonempty) {
            Ok(params) if input.is_empty() => Ok(Self(params.into_iter().collect())),
            _ => Err(input.error("expected name of generic parameter")),
        }
    }
}

impl From<TerminatedParams> for Vec<Param> {
    fn from(terminated_params: TerminatedParams) -> Self {
        terminated_params.0
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Param {
    Lifetime(Lifetime),
    TypeOrConst(Ident),
}

impl Parse for Param {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input
            .parse::<Lifetime>()
            .map(Self::Lifetime)
            .or_else(|_| input.parse::<Ident>().map(Self::TypeOrConst))
            .map_err(|_| input.error("expected name of generic parameter"))
    }
}

impl ToTokens for Param {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Lifetime(lifetime) => lifetime.to_tokens(tokens),
            Self::TypeOrConst(ident) => ident.to_tokens(tokens),
        }
    }
}

impl PartialEq<GenericParam> for Param {
    fn eq(&self, other: &GenericParam) -> bool {
        match (self, other) {
            (Self::Lifetime(l), GenericParam::Lifetime(LifetimeDef { lifetime: r, .. })) => l == r,
            (Self::TypeOrConst(l), GenericParam::Type(TypeParam { ident: r, .. })) => l == r,
            (Self::TypeOrConst(l), GenericParam::Const(ConstParam { ident: r, .. })) => l == r,
            _ => false,
        }
    }
}

pub fn assert_no_opts(attrs: &[Attribute], name: &str) -> Result<(), Error> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident(&ATTR_IDENT))
        .map(|attr| {
            syn::Error::new_spanned(
                attr,
                format!(
                    "#[{}] helper attribute is not supported for {}",
                    ATTR_IDENT, name
                ),
            )
        })
        .collect::<Error>()
        .ok()
}