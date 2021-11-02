use crate::dependency::DependencyOnExt;
use crate::macros::debug_assert_parse;
use crate::template::TypeTemplate;
use crate::type_param::TypeParamExt;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use std::{collections::HashSet, iter};
use syn::{
    parse_quote, AngleBracketedGenericArguments, Expr, GenericArgument, Index, Path, PathArguments,
    PathSegment, QSelf, Type, TypeArray, TypeParam, TypePath, TypeTuple, WherePredicate,
};

pub struct StructMapper<'a> {
    type_param: &'a TypeParam,
    src_type_param: &'a TypeParam,
    dst_type_param: &'a TypeParam,
    where_predicates: HashSet<WherePredicate>,
}

impl<'a> StructMapper<'a> {
    pub fn new(
        type_param: &'a TypeParam,
        src_type_param: &'a TypeParam,
        dst_type_param: &'a TypeParam,
    ) -> Self {
        Self {
            type_param,
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

    pub fn map_struct(&mut self, mappable: TokenStream, ty: &Type) -> TokenStream {
        debug_assert_parse!(mappable as Expr);

        if ty.dependency_on(self.type_param).is_none() {
            return mappable;
        }

        match ty {
            Type::Array(TypeArray { elem: inner_ty, .. }) => {
                let template = TypeTemplate::new(ty, self.type_param);
                let src_type = template.apply(self.src_type_param.to_type());
                let dst_type = template.apply(self.dst_type_param.to_type());

                let inner_template = TypeTemplate::new(inner_ty, self.type_param);
                let inner_src_type = inner_template.apply(self.src_type_param.to_type());
                let inner_dst_type = inner_template.apply(self.dst_type_param.to_type());

                self.where_predicates.insert(parse_quote! {
                    #src_type: ::mapstruct::MapStruct<#inner_src_type, #inner_dst_type, Output = #dst_type>
                });

                let closure = self.map_struct_closure(inner_ty);

                quote! {
                    #mappable.map_struct(#closure)
                }
            }
            Type::Path(TypePath {
                qself,
                path:
                    Path {
                        leading_colon,
                        segments,
                    },
            }) => {
                if let Some(QSelf { ty, .. }) = &qself {
                    if let Some(dep_ty) = ty.dependency_on(self.type_param) {
                        abort!(dep_ty, "mapping over self type is not supported");
                    }
                }

                if segments.is_empty() {
                    abort!(segments, "mapping over empty type is not supported");
                }

                let (prefix, PathSegment { ident, arguments }) = {
                    let mut segments: Vec<_> = segments.into_iter().collect();
                    let last = segments.pop().unwrap();
                    (segments, last)
                };

                if let Some(prefix_dep) = prefix
                    .iter()
                    .find_map(|segment| segment.dependency_on(self.type_param))
                {
                    abort!(
                        prefix_dep,
                        "mapping over types with associated items is not supported"
                    );
                }

                let AngleBracketedGenericArguments {
                    args,
                    colon2_token,
                    lt_token,
                    gt_token,
                } = match arguments {
                    PathArguments::None => return quote!(f(#mappable)),
                    PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed,
                    PathArguments::Parenthesized(..) => {
                        abort!(arguments, "mapping over function types is not supported")
                    }
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
                    .filter(|(_, (_, arg_type))| arg_type.dependency_on(self.type_param).is_some())
                {
                    let inner_template = TypeTemplate::new(arg_type, self.type_param);
                    let inner_src_type = inner_template.apply(self.src_type_param.to_type());
                    let inner_dst_type = inner_template.apply(self.dst_type_param.to_type());

                    let make_type = |idx: usize| {
                        let src_args = args.iter().enumerate().map(|(i, arg)| match arg {
                            GenericArgument::Type(ty) => {
                                let template = TypeTemplate::new(ty, self.type_param);

                                GenericArgument::Type(template.apply(if i >= idx {
                                    self.src_type_param.to_type()
                                } else {
                                    self.dst_type_param.to_type()
                                }))
                            }
                            _ => arg.clone(),
                        });

                        Type::Path(TypePath {
                            qself: qself.clone(),
                            path: Path {
                                leading_colon: leading_colon.clone(),
                                segments: prefix
                                    .iter()
                                    .map(|segment| PathSegment::clone(segment))
                                    .chain(iter::once(PathSegment {
                                        ident: ident.clone(),
                                        arguments: PathArguments::AngleBracketed(
                                            AngleBracketedGenericArguments {
                                                colon2_token: colon2_token.clone(),
                                                lt_token: lt_token.clone(),
                                                gt_token: gt_token.clone(),
                                                args: src_args.collect(),
                                            },
                                        ),
                                    }))
                                    .collect(),
                            },
                        })
                    };

                    let src_type = make_type(*idx);
                    let dst_type = make_type(*idx + 1);

                    self.where_predicates.insert(parse_quote! {
                        #src_type: ::mapstruct::MapStruct<#inner_src_type, #inner_dst_type, ::mapstruct::TypeParam<#type_idx>, Output = #dst_type>
                    });

                    let closure = self.map_struct_closure(arg_type);

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
                    self.map_struct(mappable, ty)
                });

                quote! {
                    (#(#mapped),*)
                }
            }
            _ => abort!(ty, "type not supported"),
        }
    }

    fn map_struct_closure(&mut self, ty: &Type) -> TokenStream {
        let closure_arg = Ident::new("value", Span::mixed_site());
        let mapped = self.map_struct(closure_arg.clone().into_token_stream(), ty);
        quote!(|#closure_arg| #mapped)
    }
}
