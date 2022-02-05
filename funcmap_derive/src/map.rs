use crate::derivable::Derivable;
use crate::ident::{MARKER_TYPE_IDENT, OUTPUT_TYPE_IDENT};
use crate::predicates::UniquePredicates;
use crate::result::Error;
use crate::syn_ext::{DependencyOnType, SubsType};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::TypeParen;
use syn::{
    parse_quote, punctuated::Pair, AngleBracketedGenericArguments, GenericArgument, Index, Path,
    PathArguments, PathSegment, QSelf, Type, TypeArray, TypeParam, TypePath,
};

#[derive(Copy, Clone, Debug)]
pub(crate) struct Mapping<'ast> {
    pub(crate) type_param: &'ast TypeParam,
    pub(crate) src_type_ident: &'ast Ident,
    pub(crate) dst_type_ident: &'ast Ident,
    pub(crate) fn_ident: &'ast Ident,
    pub(crate) crate_path: &'ast Path,
    pub(crate) derivable: Derivable,
}

impl Mapping<'_> {
    pub(crate) fn map(
        self,
        mappable: impl ToTokens,
        ty: &Type,
    ) -> Result<(TokenStream, UniquePredicates), Error> {
        let mut mapper = Mapper::new(self);
        let mapped_tokens = mapper.map(mappable.into_token_stream(), ty)?;
        Ok((mapped_tokens, mapper.unique_predicates))
    }
}

#[derive(Debug)]
struct Mapper<'ast> {
    mapping: Mapping<'ast>,
    unique_predicates: UniquePredicates,
}

impl<'ast> Mapper<'ast> {
    fn new(mapping: Mapping<'ast>) -> Self {
        Self {
            mapping,
            unique_predicates: UniquePredicates::new(),
        }
    }

    fn map(&mut self, mappable: TokenStream, ty: &Type) -> Result<TokenStream, Error> {
        if let Type::Macro(..) = ty {
            return Err(syn::Error::new_spanned(
                ty,
                // this is literally the same error message
                // that would be emitted by builtin derive macros
                "`derive` cannot be used on items with type macros",
            )
            .into());
        }

        if ty
            .dependency_on_type(&self.mapping.type_param.ident)
            .is_none()
        {
            self.unique_predicates.add(parse_quote! {
                #ty: ::core::marker::Sized
            })?;

            return Ok(mappable);
        }

        let crate_path = self.mapping.crate_path;
        let trait_ident = self.mapping.derivable.trait_ident();
        let fn_ident = self.mapping.derivable.fn_ident();

        match ty {
            Type::Array(TypeArray { elem: inner_ty, .. }) => {
                let closure = self.map_closure(inner_ty)?;
                Ok(self
                    .mapping
                    .derivable
                    .bind_expr(quote!(#crate_path::#trait_ident::#fn_ident(#mappable, #closure))))
            }

            Type::Paren(TypeParen { elem: inner_ty, .. }) => self.map(mappable, inner_ty),

            Type::Path(type_path) => {
                let TypePath {
                    qself,
                    path:
                        Path {
                            leading_colon,
                            segments,
                        },
                } = type_path;

                if let Some(QSelf { ty: inner_ty, .. }) = &qself {
                    if inner_ty
                        .dependency_on_type(&self.mapping.type_param.ident)
                        .is_some()
                    {
                        return Err(syn::Error::new_spanned(
                            ty,
                            "mapping over type with associated item is not supported",
                        )
                        .into());
                    }
                }

                let (prefix, ident, arguments) = {
                    let mut prefix = segments.clone();

                    match prefix.pop() {
                        Some(Pair::End(PathSegment { ident, arguments })) => {
                            (prefix, ident, arguments)
                        }
                        Some(..) => {
                            return Err(syn::Error::new_spanned(
                                ty,
                                "mapping over type with trailing :: is not supported",
                            )
                            .into());
                        }
                        None => {
                            return Err(syn::Error::new_spanned(
                                ty,
                                "mapping over empty type is not supported",
                            )
                            .into());
                        }
                    }
                };

                let prefix_type = Type::Path(TypePath {
                    qself: qself.clone(),
                    path: Path {
                        leading_colon: *leading_colon,
                        segments: prefix.clone(),
                    },
                });

                if prefix_type
                    .dependency_on_type(&self.mapping.type_param.ident)
                    .is_some()
                {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "mapping over type with associated item is not supported",
                    )
                    .into());
                }

                let angle_bracketed = match arguments {
                    PathArguments::None => {
                        let mapping_fn_ident = self.mapping.fn_ident;
                        return Ok(self
                            .mapping
                            .derivable
                            .bind_expr(quote!(#mapping_fn_ident(#mappable))));
                    }

                    PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed,

                    PathArguments::Parenthesized(..) => {
                        return Err(syn::Error::new_spanned(
                            ty,
                            "mapping over function type is not supported",
                        )
                        .into());
                    }
                };

                let args = angle_bracketed.args;

                let arg_types = args.iter().filter_map(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                });

                let mut mappable = mappable;

                for (type_idx, arg_type) in arg_types.enumerate() {
                    if arg_type
                        .dependency_on_type(&self.mapping.type_param.ident)
                        .is_none()
                    {
                        continue;
                    }

                    let (inner_src_type, inner_dst_type) = self.subs_types(arg_type.clone());

                    let make_type = |mapped_until_idx: usize| {
                        let mapped_args = args.iter().cloned().scan(0, |type_arg_idx, arg| {
                            Some(match arg {
                                GenericArgument::Type(ty) => {
                                    let mapped_ty = if *type_arg_idx >= mapped_until_idx {
                                        self.subs_src_type(ty)
                                    } else {
                                        self.subs_dst_type(ty)
                                    };
                                    *type_arg_idx += 1;
                                    GenericArgument::Type(mapped_ty)
                                }
                                _ => arg,
                            })
                        });

                        Type::Path(TypePath {
                            qself: qself.clone(),
                            path: Path {
                                leading_colon: *leading_colon,
                                segments: prefix
                                    .iter()
                                    .cloned()
                                    .chain([PathSegment {
                                        ident: ident.clone(),
                                        arguments: PathArguments::AngleBracketed(
                                            AngleBracketedGenericArguments {
                                                args: mapped_args.collect(),
                                                ..angle_bracketed
                                            },
                                        ),
                                    }])
                                    .collect(),
                            },
                        })
                    };

                    let src_type = make_type(type_idx);
                    let dst_type = make_type(type_idx + 1);

                    self.unique_predicates.add(parse_quote! {
                        #src_type: #crate_path::#trait_ident<
                            #inner_src_type,
                            #inner_dst_type,
                            #crate_path::#MARKER_TYPE_IDENT<#type_idx>,
                            #OUTPUT_TYPE_IDENT = #dst_type
                        >
                    })?;

                    let closure = self.map_closure(arg_type)?;

                    mappable = self.mapping.derivable.bind_expr(quote! {
                        #crate_path::#trait_ident::<
                            _,
                            _,
                            #crate_path::#MARKER_TYPE_IDENT::<#type_idx>
                        >::#fn_ident(#mappable, #closure)
                    });
                }

                Ok(mappable)
            }

            Type::Tuple(type_tuple) => {
                let mapped = type_tuple
                    .elems
                    .iter()
                    .enumerate()
                    .map(|(i, ty)| {
                        let idx = Index::from(i);
                        let mappable = quote!(#mappable.#idx);
                        self.map(mappable, ty)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(quote!((#(#mapped),*)))
            }

            Type::BareFn(..) => Err(syn::Error::new_spanned(
                ty,
                "mapping over function type is not supported",
            )
            .into()),

            Type::Ptr(..) => Err(syn::Error::new_spanned(
                ty,
                "mapping over pointer type is not supported",
            )
            .into()),

            Type::Reference(..) => Err(syn::Error::new_spanned(
                ty,
                "mapping over reference type is not supported",
            )
            .into()),

            Type::Slice(..) => {
                Err(syn::Error::new_spanned(ty, "mapping over slice type is not supported").into())
            }

            Type::TraitObject(..) => Err(syn::Error::new_spanned(
                ty,
                "mapping over trait object type is not supported",
            )
            .into()),

            _ => Err(syn::Error::new_spanned(ty, "mapping over this type is not supported").into()),
        }
    }

    fn map_closure(&mut self, ty: &Type) -> Result<TokenStream, Error> {
        let closure_arg = Ident::new("value", Span::mixed_site());
        let mapped = self.map(closure_arg.clone().into_token_stream(), ty)?;
        let expr = self.mapping.derivable.unit_expr(mapped);
        Ok(quote!(|#closure_arg| #expr))
    }

    fn subs_src_type(&self, ty: Type) -> Type {
        ty.subs_type(&self.mapping.type_param.ident, self.mapping.src_type_ident)
    }

    fn subs_dst_type(&self, ty: Type) -> Type {
        ty.subs_type(&self.mapping.type_param.ident, self.mapping.dst_type_ident)
    }

    fn subs_types(&self, ty: Type) -> (Type, Type) {
        (self.subs_src_type(ty.clone()), self.subs_dst_type(ty))
    }
}
