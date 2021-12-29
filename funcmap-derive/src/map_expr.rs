use crate::idents::*;
use crate::predicates::UniquePredicates;
use crate::syn_ext::{DependencyOnType, SubsType};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{diagnostic, Diagnostic, Level};
use quote::{quote, ToTokens};
use syn::TypeParen;
use syn::{
    parse_quote, punctuated::Pair, AngleBracketedGenericArguments, GenericArgument, Index, Path,
    PathArguments, PathSegment, QSelf, Type, TypeArray, TypeParam, TypePath,
};

pub fn map_expr(
    mappable: impl ToTokens,
    ty: &Type,
    type_param: &TypeParam,
    src_type_ident: &Ident,
    dst_type_ident: &Ident,
    mapping_fn_ident: &Ident,
    crate_path: &Path,
) -> Result<(TokenStream, UniquePredicates), Diagnostic> {
    let mut mapper = Mapper {
        type_param,
        src_type_ident,
        dst_type_ident,
        mapping_fn_ident,
        crate_path,
        unique_predicates: UniquePredicates::new(),
    };

    let mapped = mapper.map(mappable.into_token_stream(), ty)?;
    Ok((mapped, mapper.unique_predicates))
}

#[derive(Debug)]
struct Mapper<'ast> {
    type_param: &'ast TypeParam,
    src_type_ident: &'ast Ident,
    dst_type_ident: &'ast Ident,
    mapping_fn_ident: &'ast Ident,
    crate_path: &'ast Path,
    unique_predicates: UniquePredicates,
}

impl<'ast> Mapper<'ast> {
    fn map(&mut self, mappable: TokenStream, ty: &Type) -> Result<TokenStream, Diagnostic> {
        if let Type::Macro(..) = ty {
            return Err(diagnostic!(
                ty,
                Level::Error,
                "`derive` cannot be used on items with type macros"
            ));
        }

        if ty.dependency_on_type(&self.type_param.ident).is_none() {
            return Ok(mappable);
        }

        let crate_path = self.crate_path;

        match ty {
            Type::Array(TypeArray { elem: inner_ty, .. }) => {
                let (src_type, dst_type) = self.subs_types(ty.clone());
                let (inner_src_type, inner_dst_type) = self.subs_types(Type::clone(inner_ty));

                self.unique_predicates.add(parse_quote! {
                    #src_type: #crate_path::#TRAIT_IDENT<
                        #inner_src_type,
                        #inner_dst_type,
                        #OUTPUT_TYPE_IDENT = #dst_type
                    >
                })?;

                let closure = self.map_closure(inner_ty)?;

                Ok(quote!(#mappable.#FN_IDENT(#closure)))
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

                if let Some(QSelf { ty, .. }) = &qself {
                    if let Some(dep_ty) = ty.dependency_on_type(&self.type_param.ident) {
                        return Err(diagnostic!(
                            dep_ty,
                            Level::Error,
                            "mapping over type with associated item is not supported"
                        ));
                    }
                }

                let (prefix, ident, arguments) = {
                    let mut prefix = segments.clone();
                    match prefix.pop() {
                        Some(Pair::End(PathSegment { ident, arguments })) => {
                            (prefix, ident, arguments)
                        }
                        Some(..) => {
                            return Err(diagnostic!(
                                segments,
                                Level::Error,
                                "mapping over type with trailing :: is not supported"
                            ))
                        }
                        None => {
                            return Err(diagnostic!(
                                segments,
                                Level::Error,
                                "mapping over empty type is not supported"
                            ))
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

                if let Some(prefix_dep) = prefix_type.dependency_on_type(&self.type_param.ident) {
                    return Err(diagnostic!(
                        prefix_dep,
                        Level::Error,
                        "mapping over type with associated item is not supported"
                    ));
                }

                let angle_bracketed = match arguments {
                    PathArguments::None => {
                        let mapping_fn_ident = self.mapping_fn_ident;
                        return Ok(quote!(#mapping_fn_ident(#mappable)));
                    }
                    PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed,
                    PathArguments::Parenthesized(..) => {
                        return Err(diagnostic!(
                            arguments,
                            Level::Error,
                            "mapping over function type is not supported"
                        ))
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
                        .dependency_on_type(&self.type_param.ident)
                        .is_none()
                    {
                        continue;
                    }

                    let (inner_src_type, inner_dst_type) = self.subs_types(arg_type.clone());

                    let make_type = |mapped_until_idx: usize| {
                        let mapped_args =
                            map_type_args(args.iter().cloned(), |type_arg_idx, ty: Type| {
                                if type_arg_idx >= mapped_until_idx {
                                    self.subs_src_type(ty)
                                } else {
                                    self.subs_dst_type(ty)
                                }
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
                        #src_type: #crate_path::#TRAIT_IDENT<
                            #inner_src_type,
                            #inner_dst_type,
                            #crate_path::#MARKER_TYPE_IDENT<#type_idx>,
                            #OUTPUT_TYPE_IDENT = #dst_type
                        >
                    })?;

                    let closure = self.map_closure(arg_type)?;

                    mappable = quote! {
                        #mappable.#FN_IDENT_WITH_MARKER(#crate_path::#MARKER_TYPE_IDENT::<#type_idx>, #closure)
                    }
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
            Type::BareFn(..) => Err(diagnostic!(
                ty,
                Level::Error,
                "mapping over function type is not supported"
            )),
            Type::Ptr(..) => Err(diagnostic!(
                ty,
                Level::Error,
                "mapping over pointer type is not supported"
            )),
            Type::Reference(..) => Err(diagnostic!(
                ty,
                Level::Error,
                "mapping over reference type is not supported"
            )),
            Type::Slice(..) => Err(diagnostic!(
                ty,
                Level::Error,
                "mapping over slice type is not supported"
            )),
            Type::TraitObject(..) => Err(diagnostic!(
                ty,
                Level::Error,
                "mapping over trait object type is not supported"
            )),
            _ => Err(diagnostic!(
                ty,
                Level::Error,
                "mapping over this type is not supported"
            )),
        }
    }

    fn map_closure(&mut self, ty: &Type) -> Result<TokenStream, Diagnostic> {
        let closure_arg = Ident::new("value", Span::mixed_site());
        let mapped = self.map(closure_arg.clone().into_token_stream(), ty)?;
        Ok(quote!(|#closure_arg| #mapped))
    }

    fn subs_src_type(&self, ty: Type) -> Type {
        ty.subs_type(&self.type_param.ident, self.src_type_ident)
    }

    fn subs_dst_type(&self, ty: Type) -> Type {
        ty.subs_type(&self.type_param.ident, self.dst_type_ident)
    }

    fn subs_types(&self, ty: Type) -> (Type, Type) {
        (self.subs_src_type(ty.clone()), self.subs_dst_type(ty))
    }
}

fn map_type_args<I, F>(iter: I, f: F) -> MapTypeArgs<I, F> {
    MapTypeArgs::new(iter, f)
}

#[derive(Debug)]
struct MapTypeArgs<I, F> {
    iter: I,
    f: F,
    type_idx: usize,
}

impl<I, F> MapTypeArgs<I, F> {
    fn new(iter: I, f: F) -> Self {
        Self {
            iter,
            f,
            type_idx: 0,
        }
    }
}

impl<I, F> Iterator for MapTypeArgs<I, F>
where
    I: Iterator<Item = GenericArgument>,
    F: FnMut(usize, Type) -> Type,
{
    type Item = GenericArgument;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.iter.next()? {
            GenericArgument::Type(ty) => {
                let arg = GenericArgument::Type((self.f)(self.type_idx, ty));
                self.type_idx += 1;
                arg
            }
            arg => arg,
        })
    }
}
