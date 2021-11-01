use crate::dependency::DependencyOnExt;
use crate::macros::debug_assert_parse;
use crate::subs_type_param;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::punctuated::Pair;
use syn::{
    parse_quote, AngleBracketedGenericArguments, Expr, GenericArgument, Index, PathArguments,
    QSelf, Type, TypeArray, TypeParam, TypePath, TypeTuple, WherePredicate,
};

pub struct StructMapper<'a> {
    src_type_param: &'a TypeParam,
    dst_type_param: &'a TypeParam,
    where_predicates: HashSet<WherePredicate>,
}

impl<'a> StructMapper<'a> {
    pub fn new(src_type_param: &'a TypeParam, dst_type_param: &'a TypeParam) -> Self {
        Self {
            src_type_param,
            dst_type_param,
            where_predicates: HashSet::new(),
        }
    }

    pub fn where_clause(&self) -> impl ToTokens {
        if self.where_predicates.is_empty() {
            quote!()
        } else {
            let where_predicates = self.where_predicates.iter();
            quote!(where #(#where_predicates,)*)
        }
    }

    pub fn map_struct(
        &mut self,
        mappable: TokenStream,
        ty: &Type,
        type_param: &TypeParam,
    ) -> TokenStream {
        debug_assert_parse!(mappable as Expr);

        if ty.dependency_on(type_param).is_none() {
            return mappable;
        }

        match ty {
            Type::Array(TypeArray { elem: inner_ty, .. }) => {
                let src_type =
                    subs_type_param::subs_type_param(ty, type_param, self.src_type_param);
                let dst_type =
                    subs_type_param::subs_type_param(ty, type_param, self.dst_type_param);
                let inner_src_type =
                    subs_type_param::subs_type_param(inner_ty, type_param, self.src_type_param);
                let inner_dst_type =
                    subs_type_param::subs_type_param(inner_ty, type_param, self.dst_type_param);

                self.where_predicates.insert(parse_quote! {
                    #src_type: ::mapstruct::MapStruct<#inner_src_type, #inner_dst_type, Output = #dst_type>
                });

                let closure = self.map_struct_closure(inner_ty, type_param);

                quote! {
                    #mappable.map_struct(#closure)
                }
            }
            Type::Path(TypePath { qself, path }) => {
                if let Some(QSelf { ty, .. }) = qself {
                    if let Some(dep_path) = ty.dependency_on(type_param) {
                        abort!(dep_path, "mapping over self type is not supported");
                    }
                }

                let (prefix, last) = {
                    let mut segments = path.segments.clone();
                    match segments.pop() {
                        Some(Pair::End(last)) => (segments, last),
                        Some(..) => abort!(
                            path.segments,
                            "mapping over type with trailing :: is not supported"
                        ),
                        None => abort!(path.segments, "mapping over empty type is not supported"),
                    }
                };

                let prefix = if prefix.is_empty() {
                    quote!()
                } else {
                    quote!(#prefix::)
                };

                let ident = last.ident;
                let args = match last.arguments {
                    PathArguments::None => return quote!(f(#mappable)),
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args, ..
                    }) => args,
                    PathArguments::Parenthesized(..) => abort!(
                        last.arguments,
                        "mapping over function types is not supported"
                    ),
                };

                let arg_types: Vec<_> = args
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, arg)| match arg {
                        GenericArgument::Type(ty) => Some((idx, ty)),
                        _ => None,
                    })
                    .collect();

                let mut mappable = mappable;

                for (type_idx, (idx, arg_type)) in arg_types
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, arg_type))| arg_type.dependency_on(type_param).is_some())
                {
                    let inner_src_type =
                        subs_type_param::subs_type_param(arg_type, type_param, self.src_type_param);
                    let inner_dst_type =
                        subs_type_param::subs_type_param(arg_type, type_param, self.dst_type_param);
                    let src_type: Type = {
                        let src_args = args.iter().enumerate().map(|(i, arg)| match arg {
                            GenericArgument::Type(ty) => {
                                GenericArgument::Type(subs_type_param::subs_type_param(
                                    ty,
                                    type_param,
                                    if i >= *idx {
                                        self.src_type_param
                                    } else {
                                        self.dst_type_param
                                    },
                                ))
                            }
                            _ => arg.clone(),
                        });

                        parse_quote! {
                            #prefix #ident<#(#src_args),*>
                        }
                    };
                    let dst_type: Type = {
                        let dst_args = args.iter().enumerate().map(|(i, arg)| match arg {
                            GenericArgument::Type(ty) => {
                                GenericArgument::Type(subs_type_param::subs_type_param(
                                    ty,
                                    type_param,
                                    if i > *idx {
                                        self.src_type_param
                                    } else {
                                        self.dst_type_param
                                    },
                                ))
                            }
                            _ => arg.clone(),
                        });

                        parse_quote! {
                            #prefix #ident<#(#dst_args),*>
                        }
                    };

                    self.where_predicates.insert(parse_quote! {
                        #src_type: ::mapstruct::MapStruct<#inner_src_type, #inner_dst_type, ::mapstruct::TypeParam<#type_idx>, Output = #dst_type>
                    });

                    let closure = self.map_struct_closure(arg_type, type_param);

                    mappable = quote! {
                        #mappable.map_struct_over(::mapstruct::TypeParam::<#type_idx>, #closure)
                    }
                }

                mappable
            }
            Type::Tuple(TypeTuple { elems, .. }) => {
                let mapped = elems.iter().enumerate().map(|(i, ty)| {
                    let idx = Index::from(i);
                    let mappable = quote!(#mappable.#idx);
                    self.map_struct(mappable, ty, type_param)
                });

                quote! {
                    (#(#mapped),*)
                }
            }
            _ => abort!(ty, "type not supported"),
        }
    }

    fn map_struct_closure(&mut self, ty: &Type, type_param: &TypeParam) -> TokenStream {
        let closure_arg = Ident::new("value", Span::mixed_site());
        let mapped = self.map_struct(closure_arg.clone().into_token_stream(), ty, type_param);
        quote!(|#closure_arg| #mapped)
    }
}
