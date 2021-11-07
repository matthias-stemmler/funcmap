use crate::type_param::TypeParamExt;
use crate::{dependency::DependencyOnExt, type_param::TypeExt};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::WhereClause;
use syn::{
    parse_quote, punctuated::Pair, AngleBracketedGenericArguments, GenericArgument, Index, Path,
    PathArguments, PathSegment, QSelf, Type, TypeArray, TypeParam, TypePath, WherePredicate,
};

pub struct StructMapper<'a> {
    type_param: &'a TypeParam,
    type_mapping: TypeMapping<'a>,
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
            type_mapping: TypeMapping::new(type_param, src_type_param, dst_type_param),
            where_predicates: HashSet::new(),
        }
    }

    pub fn where_clause(self) -> WhereClause {
        WhereClause {
            where_token: Default::default(),
            predicates: self.where_predicates.into_iter().collect(),
        }
    }

    pub fn map_struct(&mut self, mappable: TokenStream, ty: &Type) -> TokenStream {
        if ty.dependency_on(self.type_param).is_none() {
            return mappable;
        }

        match ty {
            Type::Array(TypeArray { elem: inner_ty, .. }) => {
                let (src_type, dst_type) = self.type_mapping.apply(ty);
                let (inner_src_type, inner_dst_type) = self.type_mapping.apply(inner_ty);

                self.where_predicates.insert(parse_quote! {
                    #src_type: ::mapstruct::MapStruct<#inner_src_type, #inner_dst_type, Output = #dst_type>
                });

                let closure = self.map_struct_closure(inner_ty);

                quote! {
                    #mappable.map_struct(#closure)
                }
            }
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
                    if let Some(dep_ty) = ty.dependency_on(self.type_param) {
                        abort!(dep_ty, "mapping over self type is not supported");
                    }
                }

                let (prefix, ident, arguments) = {
                    let mut prefix = segments.clone();
                    match prefix.pop() {
                        Some(Pair::End(PathSegment { ident, arguments })) => {
                            (prefix, ident, arguments)
                        }
                        Some(_) => abort!(
                            segments,
                            "mapping over type with trailing :: is not supported"
                        ),
                        None => abort!(segments, "mapping over empty type is not supported"),
                    }
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
                    colon2_token,
                    lt_token,
                    args,
                    gt_token,
                } = match arguments {
                    PathArguments::None => return quote!(f(#mappable)),
                    PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed,
                    PathArguments::Parenthesized(..) => {
                        abort!(arguments, "mapping over function types is not supported")
                    }
                };

                let arg_types = args.iter().filter_map(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                });

                let mut mappable = mappable;

                for (type_idx, arg_type) in arg_types.enumerate() {
                    if arg_type.dependency_on(self.type_param).is_none() {
                        continue;
                    }

                    let (inner_src_type, inner_dst_type) = self.type_mapping.apply(arg_type);

                    let make_type = |mapped_until_idx: usize| {
                        let args = map_type_args(args.iter(), |type_arg_idx, ty| {
                            if type_arg_idx >= mapped_until_idx {
                                self.type_mapping.apply_src(ty)
                            } else {
                                self.type_mapping.apply_dst(ty)
                            }
                        });

                        Type::Path(TypePath {
                            qself: qself.clone(),
                            path: Path {
                                leading_colon: leading_colon.clone(),
                                segments: prefix
                                    .iter()
                                    .cloned()
                                    .chain(Some(PathSegment {
                                        ident: ident.clone(),
                                        arguments: PathArguments::AngleBracketed(
                                            AngleBracketedGenericArguments {
                                                colon2_token: colon2_token.clone(),
                                                lt_token: lt_token.clone(),
                                                args: args.collect(),
                                                gt_token: gt_token.clone(),
                                            },
                                        ),
                                    }))
                                    .collect(),
                            },
                        })
                    };

                    let src_type = make_type(type_idx);
                    let dst_type = make_type(type_idx + 1);

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
            Type::Tuple(type_tuple) => {
                let mapped = type_tuple.elems.iter().enumerate().map(|(i, ty)| {
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

struct TypeMapping<'a> {
    type_param: &'a TypeParam,
    src_type_param: &'a TypeParam,
    dst_type_param: &'a TypeParam,
}

impl<'a> TypeMapping<'a> {
    fn new(
        type_param: &'a TypeParam,
        src_type_param: &'a TypeParam,
        dst_type_param: &'a TypeParam,
    ) -> Self {
        Self {
            type_param,
            src_type_param,
            dst_type_param,
        }
    }

    fn apply_src(&self, ty: &Type) -> Type {
        ty.subs_type_param(self.type_param, &self.src_type_param.to_type())
    }

    fn apply_dst(&self, ty: &Type) -> Type {
        ty.subs_type_param(self.type_param, &self.dst_type_param.to_type())
    }

    fn apply(&self, ty: &Type) -> (Type, Type) {
        (self.apply_src(ty), self.apply_dst(ty))
    }
}

fn map_type_args<I, F>(iter: I, f: F) -> MapTypeArgs<I, F> {
    MapTypeArgs::new(iter, f)
}

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

impl<'a, I, F> Iterator for MapTypeArgs<I, F>
where
    I: Iterator<Item = &'a GenericArgument>,
    F: FnMut(usize, &'a Type) -> Type,
{
    type Item = GenericArgument;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.iter.next()? {
            GenericArgument::Type(ty) => {
                let arg = GenericArgument::Type((self.f)(self.type_idx, ty));
                self.type_idx += 1;
                arg
            }
            arg => arg.clone(),
        })
    }
}
